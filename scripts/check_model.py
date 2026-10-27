#!/usr/bin/env python3
"""Consistency checks for the ERMES data model (classes/, objects/, rules/).

Run from the repository root:

    python scripts/check_model.py

Only the standard library is used. Exit code is 1 if at least one error is found.
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

errors = []
warnings = []


def error(file, msg):
    errors.append(f"ERROR {file}: {msg}")


def warn(file, msg):
    warnings.append(f"WARN  {file}: {msg}")


def load_classes():
    classes = {}
    for path in sorted((ROOT / "classes").glob("*.json")):
        rel = path.relative_to(ROOT).as_posix()
        try:
            cls = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as e:
            error(rel, f"invalid JSON: {e}")
            continue
        name = cls.get("name")
        if not name:
            error(rel, "missing 'name'")
            continue
        if name in classes:
            error(rel, f"class {name} already defined in {classes[name]['_file']}")
            continue
        cls["_file"] = rel
        classes[name] = cls
    return classes


def ancestors(classes, name):
    """Returns the class itself and all its ancestors."""
    seen, queue = [], [name]
    while queue:
        current = queue.pop(0)
        if current in seen or current not in classes:
            continue
        seen.append(current)
        queue.extend(classes[current].get("parents", []))
    return seen


def properties(classes, names, kind):
    """Maps property name -> (declaring class, property) over the hierarchy of the given classes."""
    props = {}
    for name in names:
        for cls_name in ancestors(classes, name):
            for prop_name, prop in classes[cls_name].get(kind, {}).items():
                props.setdefault(prop_name, (cls_name, prop))
    return props


def check_classes(classes):
    for name, cls in classes.items():
        rel = cls["_file"]
        for parent in cls.get("parents", []):
            if parent not in classes:
                error(rel, f"parent class {parent} does not exist")
        declared = {}
        for kind in ("static_properties", "dynamic_properties"):
            for prop_name, prop in cls.get(kind, {}).items():
                if prop_name in declared:
                    error(rel, f"property {prop_name} is both static and dynamic")
                declared[prop_name] = prop
                if prop.get("type") in ("object", "object[]"):
                    for target in prop.get("classes", []):
                        if target not in classes:
                            error(rel, f"property {prop_name} refers to class {target}, which does not exist")
                if "min" in prop and "max" in prop and prop["min"] > prop["max"]:
                    error(rel, f"property {prop_name} has min > max")
                allowed = prop.get("allowed_values")
                if allowed is not None and "default" in prop and prop["default"] not in allowed:
                    error(rel, f"default of {prop_name} is not among its allowed values")

        # The same property inherited from different classes with different types is ambiguous.
        seen = {}
        for cls_name in ancestors(classes, name):
            for kind in ("static_properties", "dynamic_properties"):
                for prop_name, prop in classes[cls_name].get(kind, {}).items():
                    if prop_name in seen and seen[prop_name][1] != prop.get("type"):
                        warn(rel, f"property {prop_name} is {seen[prop_name][1]} in {seen[prop_name][0]} but {prop.get('type')} in {cls_name}")
                    seen.setdefault(prop_name, (cls_name, prop.get("type")))


def check_value(rel, prop_name, prop, value):
    kind = prop.get("type")
    if value is None:
        return
    expected = {"bool": bool, "int": int, "float": (int, float), "string": str, "symbol": str, "object": str}.get(kind)
    if expected and (not isinstance(value, expected) or (kind in ("int", "float") and isinstance(value, bool))):
        error(rel, f"value of {prop_name} should be {kind}, found {value!r}")
        return
    if kind == "symbol" and prop.get("allowed_values") is not None and value not in prop["allowed_values"]:
        error(rel, f"value {value!r} of {prop_name} is not among its allowed values")
    if kind in ("int", "float"):
        if "min" in prop and value < prop["min"]:
            error(rel, f"value of {prop_name} is below its minimum ({prop['min']})")
        if "max" in prop and value > prop["max"]:
            error(rel, f"value of {prop_name} is above its maximum ({prop['max']})")


def check_objects(classes):
    ids = {}
    for path in sorted((ROOT / "objects").glob("*.json")):
        rel = path.relative_to(ROOT).as_posix()
        try:
            obj = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as e:
            error(rel, f"invalid JSON: {e}")
            continue
        obj_id = obj.get("id")
        if not obj_id:
            error(rel, "missing 'id'")
        elif obj_id in ids:
            error(rel, f"id {obj_id} already used by {ids[obj_id]}")
        else:
            ids[obj_id] = rel
        obj_classes = obj.get("classes", [])
        missing = [c for c in obj_classes if c not in classes]
        for c in missing:
            error(rel, f"class {c} does not exist")
        if missing:
            continue
        static_props = properties(classes, obj_classes, "static_properties")
        for prop_name, value in obj.get("properties", {}).items():
            if prop_name not in static_props:
                error(rel, f"{prop_name} is not a static property of {', '.join(obj_classes)} (it would be ignored)")
            else:
                check_value(rel, prop_name, static_props[prop_name][1], value)


def template_property(classes, template):
    """Resolves a CLIPS template name such as 'SharedDevice_patient' into (class, property name, property)."""
    for cls_name, cls in classes.items():
        if template.startswith(cls_name + "_"):
            prop_name = template[len(cls_name) + 1 :]
            for kind in ("static_properties", "dynamic_properties"):
                if prop_name in cls.get(kind, {}):
                    return cls_name, prop_name, cls[kind][prop_name]
    return None


def check_rules(classes):
    pattern = re.compile(r"\(\s*([A-Za-z][\w-]*)\s+\(id\s+\?[\w-]+\)(.*)")
    value_var = re.compile(r"\(value\s+\?([\w-]+)")
    add_data = re.compile(r"\(add-data\s+\?([\w-]+)\s+\(create\$([^)]*)\)\s+\(create\$([^)]*)\)")
    for path in sorted((ROOT / "rules").glob("*.clp")):
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8")
        bindings = {}  # variable -> (class, property name, property)
        for line in text.splitlines():
            match = pattern.search(line)
            if not match:
                continue
            template = match.group(1)
            if template in classes:
                continue
            resolved = template_property(classes, template)
            if resolved is None:
                error(rel, f"template {template} matches no class or property declared by that class")
                continue
            var = value_var.search(match.group(2))
            if var:
                bindings[var.group(1)] = resolved

        for target_var, raw_names, raw_values in add_data.findall(text):
            target = bindings.get(target_var)
            if target is None or target[2].get("type") != "object":
                warn(rel, f"cannot infer the class of ?{target_var}, add-data not checked")
                continue
            target_classes = target[2].get("classes", [])
            target_props = properties(classes, target_classes, "dynamic_properties")
            names, values = raw_names.split(), raw_values.split()
            if len(names) != len(values):
                error(rel, "add-data has a different number of property names and values")
            for prop_name, value in zip(names, values):
                if prop_name not in target_props:
                    error(rel, f"{prop_name} is not a dynamic property of {', '.join(target_classes)}")
                    continue
                source = bindings.get(value[1:]) if value.startswith("?") else None
                if source is None:
                    continue
                source_type, target_type = source[2].get("type"), target_props[prop_name][1].get("type")
                if source_type != target_type:
                    error(rel, f"{source[0]}.{source[1]} is {source_type} but {target_props[prop_name][0]}.{prop_name} is {target_type}: add-data will reject the value")


def main():
    classes = load_classes()
    check_classes(classes)
    check_objects(classes)
    check_rules(classes)
    for line in errors + warnings:
        print(line)
    print(f"{len(classes)} classes checked: {len(errors)} errors, {len(warnings)} warnings")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())

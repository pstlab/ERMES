import { h, type VNode } from "snabbdom";
import { OffcanvasBody, Offcanvas as OffcanvasComponent } from "@ratiosolver/flick";
import { ClassesList, ObjectsList, RulesList, type coco } from "@ratiosolver/coco";

export function ERMESOffcanvas(coco: coco.CoCo): VNode {
    return OffcanvasComponent(
        OffcanvasBody([
            coco.get_classes().size > 0 ? h('label', 'Classi') : null,
            ClassesList(coco),
            coco.get_objects().size > 0 ? h('label', 'Oggetti') : null,
            ObjectsList(coco),
            coco.get_rules().size > 0 ? h('label', 'Regole') : null,
            RulesList(coco),
        ])
    );
}
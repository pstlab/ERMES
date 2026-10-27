# ERMES Classes

This folder defines the domain classes used by ERMES as JSON class descriptors.
Each file describes a class name, optional parent classes, and static/dynamic
properties with type and constraint metadata.

Legend: `S:` = static property, `D:` = dynamic property. Inherited properties
shown only in the declaring class.

```mermaid
classDiagram
	class Patient {
		+S: name string
		+S: age int
		+S: height float
		+D: weight float
		+D: systolic_blood_pressure int
		+D: diastolic_blood_pressure int
		+D: heart_rate int
		+D: fallen bool
		+D: cognitive_performance float
	}

	class Caregiver {
		+S: name string
		+S: phone string
	}

	class Residence {
		+S: name string
		+S: city string
	}

	class Room {
		+S: name string
		+S: floor int
	}

	class Sensor {
		+S: name string
	}

	class Actuator {
		+S: name string
	}

	class StaticallyLocated {
		+S: x float
		+S: y float
	}

	class DynamicallyLocated {
		+S: lat float
		+S: lng float
	}

	class RoomLocated {
		+D: room Room
	}

	class SharedDevice {
		+D: patient Patient
	}

	class BloodPressureSensor {
		+D: systolic_blood_pressure int
		+D: diastolic_blood_pressure int
		+D: heart_rate int
	}

	class CognitiveExercise {
		+D: performance float
	}

	class FallDetector {
		+D: fallen bool
		+D: confidence float
	}

	class Locator {
		+D: room symbol
	}

	class rPPG {
		+D: HR int
		+D: BR int
		+D: SpO2 int
	}

	class SWF {
		+D: Fall int
		+D: HR int
		+D: Sms int
	}

	class Seremy {
		+S: seremy_id string
		+D: alarm symbol
		+D: alarm_source symbol
		+D: min_bpm int
		+D: max_bpm int
		+D: avg_bpm int
		+D: steps int
		+D: latitude float
		+D: longitude float
	}

	class Photosensor {
		+S: location symbol
		+S: enabled bool
		+D: lux float
		+D: cct int
		+D: status symbol
	}

	class LampDevice {
		+S: role symbol
		+S: max_flux float
		+S: calibration_k float
		+S: enabled bool
		+D: dimming float
		+D: cct int
	}

	class Alert {
		+S: channel string
		+D: subject string
		+D: level string
		+D: message string
	}

	Sensor <|-- BloodPressureSensor
	Sensor <|-- CognitiveExercise
	Sensor <|-- FallDetector
	Sensor <|-- Locator
	Sensor <|-- rPPG
	Sensor <|-- SWF
	Sensor <|-- Seremy
	Sensor <|-- Photosensor
	Sensor <|-- Alert

	Actuator <|-- LampDevice

	SharedDevice <|-- BloodPressureSensor
	SharedDevice <|-- CognitiveExercise
	SharedDevice <|-- FallDetector
	SharedDevice <|-- Locator
	SharedDevice <|-- rPPG
	SharedDevice <|-- SWF
	SharedDevice <|-- Seremy

	StaticallyLocated <|-- Photosensor
	StaticallyLocated <|-- LampDevice

	DynamicallyLocated <|-- Patient
	DynamicallyLocated <|-- SWF

	RoomLocated <|-- Patient

	SharedDevice --> Patient : patient
	RoomLocated --> Room : room
```

## Devices → Patient

Devices are `SharedDevice`s; `patient` points to the assigned `Patient`.
Rules in [`../rules`](../rules) copy values to the patient via `add-data`.

| Device class | Rule | Patient property |
|---|---|---|
| `BloodPressureSensor` | `blood-pressure-measurement` | `systolic_blood_pressure`, `diastolic_blood_pressure`, `heart_rate` |
| `CognitiveExercise` | `cognitive-performance` | `cognitive_performance` |
| `FallDetector` | `fall-event` | `fallen` |
| `Locator` | `patient-locator` | `room` (rejected, see notes) |
| `rPPG`, `SWF`, `Seremy` | - | - |

CLIPS templates: one per class (`id` slot), one per property, named
`<DeclaringClass>_<property>` (e.g. `SharedDevice_patient`, `RoomLocated_room`).

## Notes

- Value types must match declared types exactly (`int` ≠ `float`, `symbol` ≠
  `object`). `add-data` produces symbols: cannot set `object` properties
  (e.g. `RoomLocated.room`).
- Startup creates classes and rules only if the name is new, objects only if
  the id is new. Edits to existing files do not reach a running deployment.
- Check: `python scripts/check_model.py`.

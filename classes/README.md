# ERMES Classes

This folder defines the domain classes used by ERMES as JSON class descriptors.
Each file describes a class name, optional parent classes, and static/dynamic
properties with type and constraint metadata.

Legend: `S:` = static property, `D:` = dynamic property.

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
        +D: room symbol
        +D: fallen boolean
        +D: cognitive_performance float
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
		+D: room symbol
	}

	class SharedDevice {
		+D: patient Patient
	}

	class AmbientLightSensor {
		+D: lux float
	}

	class BloodPressureSensor {
		+D: systolic_blood_pressure int
		+D: diastolic_blood_pressure int
		+D: heart_rate int
	}

	class Locator {
		+D: room symbol
	}

	class Alert {
		+S: channel string
		+D: subject string
		+D: level string
		+D: message string
	}

	class Caregiver {
		+S: name string
		+S: phone string
	}

	class CognitiveExercise {
		+D: performance float
	}

	class Lamp {
		+D: brightness float
		+D: temperature float
	}

	class Residence {
		+S: name string
		+S: city string
	}

	class Room {
		+S: name string
		+S: floor int
	}

	Sensor <|-- AmbientLightSensor
	Sensor <|-- Locator
	Sensor <|-- BloodPressureSensor
	Sensor <|-- CognitiveExercise

    Actuator <|-- Lamp
    Actuator <|-- Alert

	StaticallyLocated <|-- AmbientLightSensor
	StaticallyLocated <|-- Lamp
	StaticallyLocated <|-- Locator

	SharedDevice --> Patient : assigned_to
	SharedDevice <|-- BloodPressureSensor
	SharedDevice <|-- CognitiveExercise
	SharedDevice <|-- Locator

    DynamicallyLocated <|-- Patient
```

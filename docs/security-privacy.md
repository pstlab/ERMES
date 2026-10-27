# Security and privacy

## Personal data

| Where | Data | Kind |
|---|---|---|
| `Patient` | name, age, height, weight, blood pressure, heart rate, falls, cognitive performance, room, position | identifying, health, location |
| `Caregiver` | name, phone | identifying |
| `BloodPressureSensor`, `rPPG`, `SWF`, `FallDetector`, `CognitiveExercise` | vital signs, falls, cognitive scores | health |
| `Seremy` | GPS position, heart rate, steps, panic alarms | health, location |
| ReHub (`src/rehub.rs`) | patient ids, daily exercise sessions | health |

Health data: special category, art. 9 GDPR.

## Data flows

- Devices → ERMES: CoCo HTTP API (JWT), MQTT (`MQTT_BROKER`, no credentials).
- Seremy → ERMES: Google Cloud Pub/Sub. Device → object via `seremy_id`.
- ERMES → ReHub: HTTPS POST every 60 s. Body RSA-OAEP (SHA-256) with ReHub public key, nonce, timestamp. Response: plain JSON over TLS.
- Storage: MongoDB. Every value appended to `object_data`, no expiry.

## Current state

- HTTP API behind authentication.
- MongoDB, Mosquitto: not published outside the Docker network.
- `config/`, `.env`: git-ignored.
- Devices linked to patients by object id.

# Security and privacy notes

This document maps the personal data handled by the current ERMES code and
lists the measures recommended before real residents' data flows through the
system. It covers the security and data protection aspects of the ICT
architecture (OS6) and should be updated whenever a new data source is added.

## Personal data

| Where | Data | Kind |
|---|---|---|
| `Patient` | name, age, height, weight, blood pressure, heart rate, falls, cognitive performance, room, position | identifying, health, location |
| `Caregiver` | name, phone | identifying |
| `BloodPressureSensor`, `rPPG`, `SWF`, `FallDetector`, `CognitiveExercise` | vital signs, falls, cognitive scores | health |
| `Seremy` | GPS position, heart rate, steps, panic alarms | health, location |
| ReHub (`src/rehub.rs`) | patient ids and daily exercise sessions | health |

Health data are a special category under art. 9 GDPR, and the people involved
are elderly residents: the bar is higher than for an ordinary web service.

## Data flows

- **Devices to ERMES**: CoCo HTTP API (authenticated) and MQTT. The MQTT
  module connects to `MQTT_BROKER` without credentials.
- **Seremy to ERMES**: Google Cloud Pub/Sub subscription. Messages are matched
  to ERMES objects through the `seremy_id` property.
- **ERMES to ReHub**: every 60 seconds, an HTTPS POST whose body is encrypted
  with the ReHub public key (RSA-OAEP, SHA-256) and carries a nonce and a
  timestamp. The response is plain JSON over TLS.
- **Storage**: MongoDB. Every received value is appended to the `object_data`
  collection, so the whole history is kept, with no expiry.

## Already in place

- The HTTP API requires authentication.
- MongoDB and Mosquitto are not published outside the Docker network
  (`compose.yml`).
- `config/` and `.env` are git-ignored, so keys and credentials stay out of the
  repository.
- Devices are linked to a patient through object ids, not personal data.

## Recommendations

1. **Logs.** When `RUST_LOG` is not set the log level is `trace`. At that level
   `ReHubModule` logs the whole ReHub response, `SeremyModule` logs every raw
   message (including GPS position) and CoCo logs every value added by rules.
   Set `RUST_LOG=info` in production and define how long logs are kept.
2. **Accounts.** Set `JWT_SECRET` to a long random value and
   `INITIAL_ADMIN_USERNAME`/`INITIAL_ADMIN_PASSWORD` to non-default values on
   every deployment reachable from the internet (`compose.yml` sets none of
   them). Use one account per partner or integration, with the least
   privileges it needs. `/register` is open to anyone: new accounts cannot read
   objects, but self-registration is better disabled once real data is in.
3. **MQTT.** Before devices in the residences publish over MQTT, the broker
   needs authentication, TLS and per-device topic ACLs. The CoCo MQTT module
   will need to support credentials.
4. **MongoDB.** Enable authentication and encrypted backups, and restrict
   access to the ERMES service.
5. **Retention.** Agree with the clinical partners how long each kind of data
   is kept (e.g. raw values for N months, then aggregates) and enforce it, for
   instance with a TTL index on `object_data`.
6. **Minimisation.** Rules do not need the patient's name: ERMES can use a
   pseudonymous code, with the mapping to real identities kept by the clinical
   partner.
7. **Deployment secrets.** `ReHubModule` reads `REHUB_ENDPOINT_URL`,
   `REHUB_CLIENT_ID` and `REHUB_PUBLIC_KEY_CREDENTIALS` (default
   `config/rehub_public_key.pem`) and stops the process if the key file is
   missing. `SeremyModule` reads `SEREMY_SUBSCRIPTION_NAME` (or
   `GCLOUD_PROJECT` and `PUBSUB_SUBSCRIPTION`) and needs Google Cloud
   credentials. `compose.yml` does not pass these variables nor mount
   `config/`: they should be provided as deployment secrets, never baked into
   the image.
8. **GDPR formalities.** A data protection impact assessment (art. 35) and the
   controller/processor roles among partners and external providers (Google
   Cloud, ReHub) should be settled with the project's DPO before the pilot.

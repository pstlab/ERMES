(deftemplate os6_vital_signs
  (slot patient_name (type STRING))
  (slot heart_rate_bpm (type FLOAT))
  (slot respiration_rate_rpm (type FLOAT))
  (slot spo2_pct (type FLOAT))
  (slot ts (type STRING))
  (slot source (type SYMBOL)))

(deftemplate os6_fall_event
  (slot patient_name (type STRING))
  (slot confidence (type FLOAT))
  (slot ts (type STRING))
  (slot source (type SYMBOL)))

(deftemplate os6_environment
  (slot room_name (type STRING))
  (slot temperature_c (type FLOAT))
  (slot luminosity_lux (type FLOAT))
  (slot ts (type STRING))
  (slot source (type SYMBOL)))

(deftemplate os6_alert
  (slot level (type SYMBOL) (allowed-symbols INFO WARNING CRITICAL))
  (slot subject (type STRING))
  (slot message (type STRING))
  (slot ts (type STRING))
  (slot source (type SYMBOL)))

(deftemplate os6_action
  (slot kind (type SYMBOL) (allowed-symbols NOTIFY_CAREGIVER START_TELEPRESENCE ADJUST_LIGHTING))
  (slot subject (type STRING))
  (slot payload (type STRING))
  (slot ts (type STRING)))

(deffacts os6_demo_seed
  (os6_vital_signs
    (patient_name "Mario Rossi")
    (heart_rate_bpm 118.0)
    (respiration_rate_rpm 26.0)
    (spo2_pct 89.0)
    (ts "2026-04-29T16:00:00Z")
    (source wearable_bracelet))
  (os6_fall_event
    (patient_name "Mario Rossi")
    (confidence 0.94)
    (ts "2026-04-29T16:00:10Z")
    (source fall_detector))
  (os6_environment
    (room_name "Room-1A")
    (temperature_c 29.5)
    (luminosity_lux 40.0)
    (ts "2026-04-29T16:00:20Z")
    (source ambient_sensor)))

(defrule os6_demo_loaded
  =>
  (printout t "[OS6-DEMO] rules loaded (seed facts will assert on reset)." crlf))

(defrule os6_detect_hypoxemia
  (os6_vital_signs (patient_name ?p) (spo2_pct ?s&:(< ?s 92.0)) (ts ?ts) (source ?src))
  =>
  (assert (os6_alert
    (level CRITICAL)
    (subject ?p)
    (message (str-cat "Low SpO2 detected: " ?s "%"))
    (ts ?ts)
    (source ?src)))
  (assert (os6_action
    (kind NOTIFY_CAREGIVER)
    (subject ?p)
    (payload (str-cat "Check resident immediately; SpO2=" ?s))
    (ts ?ts))))

(defrule os6_detect_tachycardia
  (os6_vital_signs (patient_name ?p) (heart_rate_bpm ?hr&:(> ?hr 110.0)) (ts ?ts) (source ?src))
  =>
  (assert (os6_alert
    (level WARNING)
    (subject ?p)
    (message (str-cat "Tachycardia detected: HR=" ?hr " bpm"))
    (ts ?ts)
    (source ?src))))

(defrule os6_detect_fall
  (os6_fall_event (patient_name ?p) (confidence ?c&:(>= ?c 0.80)) (ts ?ts) (source ?src))
  =>
  (assert (os6_alert
    (level CRITICAL)
    (subject ?p)
    (message (str-cat "Fall detected (confidence=" ?c ")."))
    (ts ?ts)
    (source ?src)))
  (assert (os6_action
    (kind START_TELEPRESENCE)
    (subject ?p)
    (payload "Connect telepresence robot to resident + caregiver station.")
    (ts ?ts))))

(defrule os6_detect_low_light
  (os6_environment (room_name ?r) (luminosity_lux ?lux&:(< ?lux 80.0)) (ts ?ts) (source ?src))
  =>
  (assert (os6_alert
    (level INFO)
    (subject ?r)
    (message (str-cat "Low ambient light (" ?lux " lux). Consider adjustment."))
    (ts ?ts)
    (source ?src)))
  (assert (os6_action
    (kind ADJUST_LIGHTING)
    (subject ?r)
    (payload (str-cat "Increase lighting to comfort range; measured=" ?lux " lux"))
    (ts ?ts))))

(defrule os6_print_alerts
  ?a <- (os6_alert (level ?lvl) (subject ?subj) (message ?msg) (ts ?ts) (source ?src))
  =>
  (printout t "[OS6-ALERT] " ?lvl " subject=" ?subj " ts=" ?ts " source=" ?src " msg=\"" ?msg "\"" crlf))

(defrule os6_print_actions
  ?a <- (os6_action (kind ?k) (subject ?subj) (payload ?payload) (ts ?ts))
  =>
  (printout t "[OS6-ACTION] " ?k " subject=" ?subj " ts=" ?ts " payload=\"" ?payload "\"" crlf))


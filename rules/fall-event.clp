(defrule fall-event
    (FallDetector (id ?id))
    (SharedDevice_patient (id ?id) (value ?patient&~nil))
    (FallDetector_fallen (id ?id) (value ?fallen))
    =>
    (add-data ?patient (create$ fallen) (create$ ?fallen))
)
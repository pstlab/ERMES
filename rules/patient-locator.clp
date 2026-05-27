(defrule patient-locator
    (Locator (id ?id))
    (SharedDevice_patient (id ?id) (value ?patient&~nil))
    (Locator_room (id ?id) (value ?room))
    =>
    (add-data ?patient (create$ room) (create$ ?room))
)
(defrule blood-pressure-measurement
    (BloodPressureSensor (id ?id))
    (SharedDevice_patient (id ?id) (value ?patient&~nil))
    (BloodPressureSensor_systolic_blood_pressure (id ?id) (value ?systolic))
    (BloodPressureSensor_diastolic_blood_pressure (id ?id) (value ?diastolic))
    (BloodPressureSensor_heart_rate (id ?id) (value ?heart_rate))
    =>
    (add-data ?patient (create$ systolic_blood_pressure diastolic_blood_pressure heart_rate) (create$ ?systolic ?diastolic ?heart_rate))
)
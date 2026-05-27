(defrule cognitive-performance
    (CognitivePerformance (id ?id))
    (SharedDevice_patient (id ?id) (value ?patient&~nil))
    (CognitivePerformance_performance (id ?id) (value ?performance))
    =>
    (add-data ?patient (create$ cognitive_performance) (create$ ?performance))
)
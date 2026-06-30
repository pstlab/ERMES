(defrule cognitive-performance
    (CognitiveExercise (id ?id))
    (SharedDevice_patient (id ?id) (value ?patient&~nil))
    (CognitiveExercise_performance (id ?id) (value ?performance))
    =>
    (add-data ?patient (create$ cognitive_performance) (create$ ?performance))
)
# Concept Modeling Guide

## Concept Versus Implementation

    Concept:
    Garden

    Possible implementations:
    GardenRecord
    GardenRepository
    /api/gardens
    gardens table

The technical realizations may change without changing the concept.

## Relationship Semantics

    vague:
    Garden relates to Artifact

    precise:
    Garden contains Knowledge Artifact

## Common Failure Modes

- mirroring class diagrams
- treating tables as entities
- circular definitions
- untracked synonym replacement
- concepts created without evidence

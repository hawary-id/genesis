# Genesis Failure Criteria

## 1. Purpose
This document establishes the formal validation rules and scientific criteria for classifying failures in the Genesis Artificial Civilization Engine. It acts as the final gatekeeper for code inclusion and milestone validation. Its purpose is to protect the scientific integrity of the simulation by preventing architectural regressions, fake emergence, determinism breaks, evolutionary dead weight, and invalid experimental conclusions.

---

## 2. Guiding Principles
Development in Genesis is guided by the fundamental principles laid out in [GENESIS_CONSTITUTION.md](file:///c:/Genesis/docs/GENESIS_CONSTITUTION.md) and [EMERGENCE_AUDIT.md](file:///c:/Genesis/docs/EMERGENCE_AUDIT.md). These translate to four core rules for evaluating failures:

1. **Non-Guaranteed Emergence:** The simulation does not guarantee the emergence of any macro-level phenomenon (such as trade, language, or culture). The absence of an emergent phenomenon is a valid scientific outcome. It is never a system failure.
2. **Build Conditions, Not Outcomes:** Systems must strictly define local physical rules and boundary constraints, never target behaviors. Outcome injection, scripting, and teleological shortcuts invalidate the simulation.
3. **Pressure-Driven Development:** A subsystem is invalid if it operates in a vacuum where its constraints have no influence on survival, replication, or environmental state changes.
4. **Implementation Neutrality:** These criteria evaluate conceptual, information-theoretic, and scientific patterns. They are independent of specific programming languages, ECS patterns, frameworks, or storage models, remaining valid for a 10+ year development horizon.

---

## 3. Failure Categories

Every validation failure in Genesis falls into one of eight categories:

### I. Invariant Failure
A system fails when it violates physical laws, mathematical constraints, or logical limits defined for the simulation. This includes mass/energy conservation, impossible spatial jumps, or infinite updates.

### II. Determinism Failure
A system fails when execution outcomes diverge across runs using the same seed and configurations, or when snapshot serialization does not yield bit-perfect continuation.

### III. False Success Failure
A system fails when it uses developer shortcuts, hardcoding, state-copying, teleological rewards, or oracles to force a desired emergent outcome.

### IV. Pressure Deficit Failure
A system fails when the underlying environment lacks the constraints (such as resource scarcity or carrying capacity) required to make adaptation and progression dynamically necessary.

### V. Architectural Dead-End Failure
A system fails when it compiles, runs, and serializes correctly, but is functionally isolated from the simulation's selective feedback loops, remaining evolutionary inert.

### VI. Evolutionary Leverage Failure
A system fails when its presence does not expand the population's adaptive survival efficiency or fails to serve as the necessary prerequisite substrate to enable downstream emergence.

### VII. Experimental Validity Failure
A system fails when scientific conclusions are drawn from insufficient observations, single world seeds, or lack appropriate comparison baselines.

### VIII. Measurement Failure
A system fails when the observational, telemetry, visualization, or analytical methods used to evaluate a phenomenon are incapable of accurately measuring that phenomenon. This includes:
* proxy metrics that are decoupled from the underlying phenomenon
* visualization artifacts
* telemetry distortions
* diagnostic side-effects
* observer-induced execution changes
* analytical methods that cannot distinguish genuine emergence from noise

---

## 4. False Success Protection

To prevent self-deception, the simulation must reject any run that achieves a target behavior through illegal shortcuts. A run is classified as a False Success if:

1. **Information Oracles are Utilized:** Adaptive units access, query, or act on environmental or social state variables outside their local observation range or state boundaries.
2. **Physical Adjacency is Bypassed:** Resources, actions, or messages are moved across spatially contiguous boundaries in a single step without traversing intermediate spaces.
3. **Teleological Utility is Injected:** Adaptive units receive survival bonuses, operational discounts, or resource boosts simply because they engage in target behaviors (e.g., communication or trade).
4. **Centralized Controllers Resolve Actions:** Coordination occurs through centralized matching engines or global resource brokers rather than local, peer-to-peer, constraint-delimited interactions.
5. **Dynamic Adaptation is Overridden:** Adaptive units transition into cooperative groups or specialized roles via hardcoded state transitions rather than dynamic, history-driven interactions.

---

## 5. False Failure Protection

Conversely, the simulation must not be marked as failing when it produces natural, albeit undesirable, evolutionary outcomes:

1. **Extinction Events:** If an adaptive lineage dies off due to extreme climate shifts or nutrient depletion, this is a valid simulation outcome. The correct action is to analyze the limiting factors, not to lower operational depletion rates.
2. **Evolutionary Stagnation:** If a population survives but fails to develop trade, language, or complex technology, this is not a code failure. It proves that the substrate did not generate sufficient selective pressure.
3. **Genetic Drift and Neutral Trait Accumulation:** The spatial clustering of non-adaptive traits due to random inheritance is a natural evolutionary baseline. It must not be treated as a bug or fake emergence unless it is hardcoded.

---

## 6. Experimental Validity Standards

To verify any emergent phenomenon, the experiment must meet the following minimum scientific baseline:

1. **Multi-Seed Verification:** The phenomenon must be observed across at least five distinct, randomly generated world seeds.
2. **Statistical Significance:** The macro-behavior must maintain a stable or growing population distribution over at least ten selective cycles.
3. **Pressure-Response Correlation:** The behavior must dissolve when the corresponding environmental pressure is relaxed (e.g., trade must disappear when resources are set to infinite replenishment).
4. **Control Group Comparison:** The adaptive behavior must show a statistically significant fitness advantage when compared to a knockout run where the subsystem is deactivated.

---

## 7. Evolutionary Leverage Validation

To verify that a subsystem has evolutionary leverage, it must be evaluated against the five classes of leverage:

```
[ Subsystem under Audit ]
           |
           +---> 1. Direct Leverage           (Immediate utility in adaptation cycles)
           +---> 2. Indirect Leverage         (Environment-mediated feedback loops)
           +---> 3. Latent Leverage           (Crisis-response survival buffer)
           +---> 4. Enabling Leverage         (Prerequisite structural pathway)
           +---> 5. Transformational Leverage  (Qualitative change in evolutionary rules)
```

1. **Direct Leverage:** The system immediately improves individual resource dynamics, traversal, or harvesting metrics. Evaluated via short-term system deactivation trials.
2. **Indirect Leverage:** The system modifies local spatial or environmental states, which improves lineage fitness over selective cycles. Evaluated via mid-term system deactivation trials.
3. **Latent Leverage:** The system provides zero benefit under stable conditions but acts as a survival buffer during environmental crashes. Evaluated via stress-test system deactivation runs (introducing extreme resource or climate crashes).
4. **Enabling Leverage:** The system is a necessary pathway for downstream roadmap phases, even if currently neutral or operationally costly. Evaluated by checking if downstream emergent behaviors are blocked when this subsystem is disabled.
5. **Transformational Leverage:** The system shifts the evolutionary paradigm of the engine (e.g., moving from genetic drift to lifetime cognitive learning, or from individual memory to cultural language). Evaluated by confirming the introduction of new optimization vectors.

---

## 8. Architectural Dead-End Detection

A subsystem is classified as an Architectural Dead-End and rejected when it operates in functional isolation.

Detection is governed by two rules:

1. **Behavioral Influence Requirement**
   Any information, state changes, or constraints generated by a subsystem must propagate in ways that influence future behaviors, environmental states, adaptation pathways, or selective pressures.
   Subsystems that generate internal representations without causal influence on future adaptive outcomes are invalid.

2. **Finite Constraint Requirement**
   Systems that influence adaptive outcomes must operate under finite constraints and trade-offs.
   Subsystems capable of generating unlimited influence without corresponding limiting factors are invalid.

---

## 9. Failure Classification Matrix

| Failure Category | Severity | Why It Matters | Detection Approach | Typical Examples | Required Action |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Invariant Failure** | Critical | Breaks physical logic; violates mass and energy conservation. | Core physical invariant validation checks. | Negative energy stocks; out-of-bounds spatial coordinates. | Halt simulation; redesign system logic immediately. |
| **Determinism Failure** | Critical | Breaks replayability; invalidates scientific peer review and snapshot persistence. | Snapshot restoration and replication comparison tests. | Execution ordering variations; unseeded runtime entropy. | Locate non-deterministic source; enforce stable sorting. |
| **False Success Failure** | High | Replaces emergent dynamics with scripted shortcuts; invalidates simulation. | Emergence audit checklists; spatial limit validation. | Querying distant entity state; instantaneous non-local state transitions. | Reject implementation; rewrite code to enforce local bounds. |
| **Pressure Deficit Failure** | Medium | Removes the driving force behind natural selection, halting progression. | Resource density telemetry; carrying capacity audits. | Unlimited resource replenishment; zero operational depletion rates. | Adjust world configuration ranges; enforce constraints. |
| **Architectural Dead-End** | High | Wastes computational budget on evolutionary inert calculations. | Dependency audits; informational coupling analysis. | Storing social relational variables that do not influence agent choices. | Reject code; couple variables to behaviors or prune features. |
| **Evolutionary Leverage Failure** | High | Halts progression along the developmental roadmap. | Dual-path leverage testing; subsystem deactivation trials. | Emitting signals that fail to coordinate spatial searching. | Restructure info transmission or increase pressure constraints. |
| **Experimental Validity Failure** | Medium | Leads to scientific self-deception and false peer conclusions. | Multi-seed testing; baseline comparative runs. | Proving trade based on a single seed over short intervals. | Reject milestone audit; rerun tests under validation standards. |
| **Measurement Failure** | High | Diagnostic errors hide bugs or create illusions of emergence. | Metric verification audits; telemetry isolation testing. | Counting random overlaps as cooperative signaling; diagnostic code altering execution speed. | Recalibrate telemetry; decouple analysis pipelines from simulation loop. |

---

## 10. Review Checklist

Reviewers must answer these questions before approving any milestone integration:

- [ ] **Are physical invariants preserved?** Verify that system updates satisfy conservation of mass, energy, and simulation physics limits.
- [ ] **Is determinism guaranteed?** Verify that simulation runs are reproducible across independent executions from identical initial seed states under deterministic execution standards.
- [ ] **Does this system bypass local constraints?** Confirm that information and resources cannot be accessed or transferred outside localized ranges or spatially contiguous boundaries.
- [ ] **Are behaviors free of teleological rewards?** Confirm that adaptive units receive no artificial utility benefits or survival bonuses for performing target behaviors.
- [ ] **Is the system functionally coupled?** Verify that the system's outputs propagate in ways that influence future behaviors or environmental states.
- [ ] **Are all outputs constrained by physical costs?** Confirm that the system operates under finite constraints and trade-offs.
- [ ] **Has the system demonstrated evolutionary leverage?** Verify the system exhibits direct, indirect, latent, enabling, or transformational leverage under subsystem deactivation testing.
- [ ] **Is the measurement valid?** Confirm that the diagnostic, visualization, and analytical tools measure the true physical state without introducing execution biases.

---

## 11. Final Decision Framework

### Question 1: When should a milestone be rejected?
A milestone must be rejected if the implementation triggers a **Critical** or **High** severity failure. This includes cases where:
1. Invariants are broken or determinism is lost.
2. Emergence is simulated through oracles, scripting, or teleological rewards.
3. The subsystem is functionally isolated (an Architectural Dead-End), fails its respective Evolutionary Leverage validation, or relies on invalid diagnostics.

### Question 2: When should a milestone be accepted even if emergence did not occur?
A milestone must be accepted if the code is implemented correctly, determinism is maintained, invariants are verified, the local constraints are functionally coupled, and the system is subject to finite constraints and trade-offs—even if the targeted macro-behavior (e.g., trade or language) did not emerge. The absence of emergence is a valid scientific run outcome.

### Question 3: How can Genesis distinguish: "A system exists" from "A system matters"?
Genesis distinguishes existence from significance by using the **Dual-Path Leverage Test**:
1. **The Coupling Path (Exists):** We verify that the system satisfies the Behavioral Influence Requirement. Its outputs propagate as constraints that influence future behaviors or environmental states.
2. **The Knockout Path (Matters):** We execute a control run (System Active) and a system deactivation run (System Disabled) under environmental stress. If the control population demonstrates a statistically significant adaptation delta or pathway progression compared to the deactivation population, the system matters.

### Question 4: How can Genesis avoid scientific self-deception?
Genesis avoids self-deception by enforcing strict boundary conditions. Developers must focus exclusively on programming the *substrate* (energy conservation, visibility limits, operational depletion rates, physical constraints). We must never program the *solutions* (alliances, dictionary structures, trading rules). If a phenomenon is not emerging naturally, we must adjust environmental pressures or document the failure, never script the outcome.

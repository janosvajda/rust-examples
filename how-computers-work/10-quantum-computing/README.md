<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 10: Quantum computing

## The idea in one sentence

A **quantum computer** controls physical systems called **qubits**, using operations that can make their probability amplitudes interfere, then measures them to obtain ordinary bits.

## Imagine… Ferris's ripple laboratory

In lesson 1, Ferris used light switches: each bit was 0 or 1.

Now imagine a pool with ripples. Two wave crests can reinforce each other. A crest and a trough can cancel. The pattern depends on how the waves line up, not just how big each wave is.

That's called **interference**. Quantum calculations work with **amplitudes**, numbers that behave a bit like those waves: each has a size, and a **phase**, like where a wave is in its up-and-down cycle. Amplitudes can add up, like two crests, or cancel out, like a crest and a trough.

A qubit isn't literally a water wave or a half-on light switch. It's a physical system (for example a single charged atom) with two chosen states, called its **basis states**, labelled **0** and **1**. The ripple picture helps explain one feature of its mathematics. [IBM's superposition and interference lesson](https://quantum.cloud.ibm.com/learning/en/modules/quantum-mechanics/superposition-with-qiskit) explores that feature.

## 1. What is different about a qubit?

**Qubit** means **quantum bit**. We write its two standard basis states as `|0⟩` and `|1⟩`; read these as “state zero” and “state one.”

| Question | Classical bit | Qubit |
|---|---|---|
| How do we describe it? | A value 0 or 1; if uncertain, probabilities for those values | A quantum state, which can be a superposition of the two basis states |
| What does a standard binary measurement give? | 0 or 1 | 0 or 1 |
| What can change an outcome's probability? | Classical operations and randomness | Quantum gates can also use interference between amplitudes |
| Does one read reveal a list of answers? | No | No |

Relative to these basis states, **superposition** means both the amplitude for 0 and the amplitude for 1 are nonzero. It doesn't mean the readout is "0.5", or that we can read both answers from one qubit: a **standard-basis measurement** gives 0 or 1.

A superposition isn't just "we don't know which it is", like a coin hidden under a cup. The coin under the cup really is heads or tails already. A qubit's amplitudes, including their phases, still take part in interference, and change what later operations do. That's the difference the next section shows. [IBM's description of quantum states](https://quantum.cloud.ibm.com/learning/en/courses/basics-of-quantum-information/single-systems/quantum-information) gives the precise model.

<details>
<summary>A little optional maths: amplitudes aren't probabilities</summary>

For an ideal **pure state** of one qubit:

```text
state = α|0⟩ + β|1⟩
probability of 0 = |α|²
probability of 1 = |β|²
|α|² + |β|² = 1
```

The Greek letters alpha and beta are **amplitudes**. In general they are complex numbers; the vertical bars here mean magnitude. Squared magnitudes are nonnegative probabilities.

For example, α = β = 1/√2 gives probabilities 1/2 and 1/2. Amplitudes 1/2 and 1/2 would *not* give a normalised state: their squared magnitudes add to only 1/2.

Noisy systems and subsystems of entangled states can need a more general description, called a **density matrix**. Our example below deliberately uses only real amplitudes for one pure-state qubit.

</details>

## 2. A gate is an operation on the state

An ideal quantum **gate** changes amplitudes in a controlled, reversible way. It isn't an ordinary logical test that first reads the qubit.

The **Hadamard gate**, written **H**, is a useful first example:

```text
prepare |0⟩ → H → measure
                  0 or 1, each with probability 1/2
```

Now apply H **twice, with no measurement in between**:

```text
prepare |0⟩ → H → H → measure
                      0 with certainty in the ideal model
```

The second H undoes the first. The amplitude for outcome 1 cancels, while the amplitude for outcome 0 remains. This is interference, not two coin tosses.

Two superpositions have special names:

```text
plus  = (|0⟩ + |1⟩) / √2     the state H makes from |0⟩
minus = (|0⟩ − |1⟩) / √2     the same sizes, but the amplitude for 1 is negative
```

Measured straight away, both give 0 or 1 with probability 1/2 each: you can't tell them apart. But apply H first, and plus always gives **0**, minus always gives **1**. The difference is the **relative phase**: the sign of one amplitude compared with the other, and interference turns it into a completely different answer. [The Hadamard gate definition](https://quantum.cloud.ibm.com/docs/en/api/qiskit/qiskit.circuit.library.HGate) specifies these transformations.

## 3. Measurement gives one result

"**Ideal**" in this lesson means a perfect, noise-free qubit; real ones are noisy (section 5). The usual measurement, which reports 0 or 1, is called a measurement in the **standard basis**. For the ideal 50/50 state, one measurement returns **one bit**, 0 or 1. The probabilities describe many freshly prepared experiments; ten shots needn't give exactly five of each.

A **shot** is one execution of the experiment. To estimate probabilities, prepare the state, run the circuit and measure again, many times.

After an ideal standard-basis measurement, this single qubit is in the basis state for the observed result. Measuring again immediately in the same basis, without intervening change, gives the same result. Measurement generally changes the state; it doesn't reveal all its amplitudes.

Putting a measurement between the two H gates changes the experiment. You can no longer rely on the final result being 0. See [IBM's measurement explanation](https://quantum.cloud.ibm.com/learning/en/courses/use-a-qc-today/quantum-mechanics-basics).

## 4. Two qubits can be entangled

Two classical bits have four possible strings: **00, 01, 10, 11**. A two-qubit state can have amplitudes associated with those four basis states.

Start with two qubits in `|00⟩`. Apply H to the first, then a **controlled-NOT**, or **CNOT**:

```text
first qubit:   |0⟩ ── H ── ● ── measure
                           │
second qubit:  |0⟩ ──────── X ── measure
```

The dot and X together are **one gate** acting on two qubits. The first qubit is the **control** (the dot), the second the **target** (the X). It flips the target from 0 to 1, or 1 to 0, when the control is 1, and leaves it alone when the control is 0. On a superposition it does both at once, for each amplitude, without measuring the control.

This prepares an **entangled** state, named a **Bell state** after the physicist John Bell:

```text
(|00⟩ + |11⟩) / √2
```

Its ideal standard-basis measurements give **00 half the time** and **11 half the time**; never 01 or 10. The joint state can't be described as two independent pure states.

Matching results alone don't prove entanglement: two ordinary coins secretly set to the same random side would also always match. Other kinds of measurement (applying gates such as H before measuring) can tell the two situations apart. [IBM's multiple-qubit lesson](https://quantum.cloud.ibm.com/learning/en/courses/basics-of-quantum-information/multiple-systems/quantum-information) explains entangled states.

Entanglement doesn't let Ada send Ben an instant message, even if their qubits are far apart. She can't choose her measurement result, and her local operations don't change the probabilities Ben sees in his own results. To compare their records and see the correlations, they need ordinary communication. [IBM's explanation of reduced states](https://quantum.cloud.ibm.com/learning/en/courses/general-formulation-of-quantum-information/density-matrices/multiple-systems) explains this limit.

## 5. What happens inside a real quantum computer?

Here is the basic **gate-based** workflow; other quantum-computing models also exist:

```text
ordinary computer builds a circuit
        ↓
compiler adapts gates to the device
        ↓
control hardware prepares and operates physical qubits
        ↓
measurement electronics produce ordinary bits
        ↓
ordinary computer collects and analyses many shots
```

The qubits must be real physical systems. Examples include:

| Technology | Where the qubit lives | How operations can be controlled |
|---|---|---|
| **Superconducting circuits** | Chosen energy states of an electrical circuit | Microwave pulses, with the chip kept extremely cold |
| **Trapped ions** | Chosen internal states of charged atoms held in a trap | Carefully controlled lasers |

The hardware doesn't execute Rust's `for` loop directly on qubits. Classical software and electronics arrange a sequence of physical operations. [IBM describes microwave-controlled circuits](https://www.ibm.com/quantum/blog/quantum-five-years); [Quantinuum documents laser-controlled ion hardware](https://docs.quantinuum.com/systems/user_guide/hardware_user_guide/h2.html).

Real operations have **noise**. Unwanted interaction with the environment can damage the state, a process called **decoherence**. **Quantum error correction** encodes a qubit's information across several physical qubits, forming a protected **logical qubit**. Extra measurements detect signs of correctable errors without reading out the logical information. It doesn't simply make backup copies: an arbitrary unknown quantum state can't be perfectly copied (section 7). **Fault-tolerant** designs also limit how faults spread during operations. With suitable error rates and enough resources, they aim to keep the failure probability small for the chosen computation; they don't promise perfect results forever. [IBM's explanation of fault tolerance](https://www.ibm.com/quantum/blog/what-is-ftqc) describes the challenge.

**Fun fact:** some quantum chips need a fridge close to absolute zero. That's a feature of those hardware designs, not a requirement for every kind of quantum computer.

## 6. Why build one?

Useful quantum algorithms arrange interference so that useful results become more likely. They don't simply “try every answer and let us read them all.”

| Example | The idea and its limits |
|---|---|
| **Simulating quantum physics** | Study quantum systems such as molecules, one motivation behind the field. Useful accuracy still needs suitable algorithms and hardware. |
| **Shor's algorithm** | Factor integers efficiently in the ideal quantum model. Factoring 15 means finding 3 × 5. Scaling to cryptographically large numbers needs a sufficiently capable, fault-tolerant machine. [IBM's Shor tutorial](https://quantum.cloud.ibm.com/docs/en/tutorials/shors-algorithm) |
| **Grover's algorithm** | Find the one right answer among N possibilities when there's no shortcut (no sorting, no pattern), using about √N checking steps instead of about N. For a million possibilities: about a thousand steps instead of up to a million. Building and running that checking operation also costs work. [IBM's query-model explanation](https://quantum.cloud.ibm.com/learning/en/courses/fundamentals-of-quantum-algorithms/grover-algorithm/unstructured-search) |

A quantum computer isn't faster for every task. Browsing a website or summing a small array doesn't automatically improve by using qubits.

A general pure-state description of **n qubits** has **2ⁿ amplitudes**, but a standard measurement of those qubits returns **one n-bit string**. A simple simulator on an ordinary computer, like the one in section 8, must store all those amplitudes. That doubles with every extra qubit: 50 qubits need about a million billion (2⁵⁰) amplitudes. Some restricted circuits permit much more efficient simulation. [Microsoft's simulator overview](https://learn.microsoft.com/en-us/azure/quantum/simulators-overview-qdk) discusses different approaches.

## 7. Where does Rust fit?

Rust can implement the **classical tools around quantum computing**: circuit representations, compilers, simulators and interfaces to hardware services.

Real examples:

- **[roqoqo](https://github.com/HQSquantumsimulations/qoqo/tree/main/roqoqo)** is HQS Quantum Simulations' Rust library for representing quantum programs and measurement information.
- **[Microsoft's Quantum Development Kit](https://github.com/microsoft/qdk)** uses Rust in its Q# tooling, including its compiler and native extension. **Q# is a different language**; Rust helps implement its tools.

An ordinary Rust `bool` is a classical value. Installing a quantum library doesn't turn the laptop's RAM into qubits.

Rust's ownership rules can help organise a software API, but they aren't the physical **no-cloning theorem**: no operation can perfectly copy an arbitrary unknown quantum state. A simulator can copy its numerical state because those numbers are ordinary classical data. [IBM's circuit-model introduction](https://quantum.cloud.ibm.com/learning/en/courses/basics-of-quantum-information/quantum-circuits/introduction) introduces that physical limit.

## 8. Try a tiny Rust simulation

This complete program models H on **one ideal qubit using real amplitudes**. It computes probabilities; it doesn't sample measurements, model entanglement or operate quantum hardware.

Save it as `quantum_demo.rs`:

```rust
use std::f64::consts::FRAC_1_SQRT_2;

// [amplitude for 0, amplitude for 1].
// These examples start normalised; H preserves that in exact arithmetic.
fn hadamard([a, b]: [f64; 2]) -> [f64; 2] {
    [(a + b) * FRAC_1_SQRT_2, (a - b) * FRAC_1_SQRT_2]
}

fn probabilities([a, b]: [f64; 2]) -> [f64; 2] {
    [a * a, b * b] // Real amplitudes only: square each one.
}

fn main() {
    let zero = [1.0, 0.0];
    let after_one = hadamard(zero);
    let after_two = hadamard(after_one);
    let minus = [FRAC_1_SQRT_2, -FRAC_1_SQRT_2];

    for (label, state) in [
        ("start", zero),
        ("one H", after_one),
        ("two H", after_two),
        ("H on minus", hadamard(minus)),
    ] {
        let [p0, p1] = probabilities(state);
        println!("{label:>10}: P(0) = {p0:.3}, P(1) = {p1:.3}");
        assert!((p0 + p1 - 1.0).abs() < 1e-12);
    }

    assert!((probabilities(after_one)[0] - 0.5).abs() < 1e-12);
    assert!((probabilities(after_two)[0] - 1.0).abs() < 1e-12);
    assert!((probabilities(hadamard(minus))[1] - 1.0).abs() < 1e-12);
}
```

From the directory containing that saved file, on macOS/Linux:

```bash
rustc quantum_demo.rs
./quantum_demo
```

On Windows PowerShell, use `rustc quantum_demo.rs`, then `.\quantum_demo.exe`.

Expected output:

```text
     start: P(0) = 1.000, P(1) = 0.000
     one H: P(0) = 0.500, P(1) = 0.500
     two H: P(0) = 1.000, P(1) = 0.000
H on minus: P(0) = 0.000, P(1) = 1.000
```

Like lesson 1's floats, these `f64` calculations are rounded. The assertions allow a tiny numerical difference.

Our simulator can inspect both amplitudes because it stores them as numbers. That isn't something one measurement of a physical qubit reveals.

## Words to remember

| Word | Meaning |
|---|---|
| **qubit** | a quantum bit: a physical system with two basis states, 0 and 1 |
| **amplitude** | a number for each possible outcome; its size squared gives the probability |
| **phase** | an amplitude's "direction", for example its sign; relative phases affect interference |
| **superposition** | here, a state with nonzero amplitudes for both 0 and 1 in the chosen basis |
| **interference** | amplitudes adding up or cancelling, like waves |
| **gate** | an ideal reversible operation on one or more qubits, such as H or CNOT |
| **measurement** | a standard-basis readout gives 0 or 1; it generally changes the state, but needn't |
| **shot** | one run of a quantum experiment, ending in measurement |
| **entanglement** | qubits sharing one joint state that can't be split into separate states |
| **decoherence** | a quantum state being spoiled by its surroundings |

This lesson has a README example rather than a Cargo package. The standalone `rustc` commands above run it; `cargo run` from this folder doesn't select a quantum lesson binary.

## A bit of history

| When | What happened, and why it mattered |
|---|---|
| **1981** | At the Physics of Computation conference, Richard Feynman argued for computers using quantum physics to simulate quantum nature. [IBM, a conference co-organiser, recalls the meeting](https://research.ibm.com/blog/qc40-physics-computation). |
| **1994** | Peter Shor developed his factoring algorithm, showing an important computational application beyond physics simulation. [IBM's account](https://quantum.cloud.ibm.com/docs/en/tutorials/shors-algorithm) |
| **1996** | Lov Grover presented his quantum search algorithm, showing a different kind of advantage: a quadratic reduction in checking operations. [Grover's original paper](https://arxiv.org/abs/quant-ph/9605043) |
| **2001** | An IBM-led experiment demonstrated Shor's algorithm factoring **15** with an NMR quantum device, which used the nuclei (centres) of atoms in specially designed molecules, dissolved in a liquid, as qubits, and controlled them with radio-frequency pulses. NMR, *nuclear magnetic resonance*, is the same physics that hospital MRI scanners use. A small demonstration is a milestone, not proof of a practical large-number factoring machine. [IBM's account of the experiment](https://www.ibm.com/quantum/blog/factor-15-shors-algorithm) |

**Fun fact:** the famous example 15 = 3 × 5 is easy for a child. Its historical interest was making a quantum algorithm work in a physical experiment.

## Your turn

**Ferris applies H to `|0⟩`, then H again, without measuring between them. Is the final ideal measurement a coin toss?**

<details>
<summary>Show the answer</summary>

No. H twice restores `|0⟩`, so the result is **0**. The intermediate superposition doesn't mean every later result remains random.

</details>

Previous: [Lesson 9: Why Rust](../09-why-rust/) · Back to the [course overview](../)

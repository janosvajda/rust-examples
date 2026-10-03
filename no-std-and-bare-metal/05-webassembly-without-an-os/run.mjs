// Loads the bare WebAssembly module with Node.js and calls its exports,
// exactly as a web page would. Also checks every result, so this script
// doubles as the lesson's test: it exits with an error if anything is wrong.
//
// Build first:  cargo build --release
// Then run:     node run.mjs

import { readFile, stat } from "node:fs/promises";
import assert from "node:assert/strict";

const path = new URL(
  "./target/wasm32-unknown-unknown/release/bare_wasm.wasm",
  import.meta.url,
);
const bytes = await readFile(path);
const { size } = await stat(path);

// The second argument is the IMPORTS object: everything the module may ask
// the outside world for. It's empty: the module needs NOTHING from us.
const { instance, module } = await WebAssembly.instantiate(bytes, {});
const exports = instance.exports;

console.log(`bare_wasm.wasm: ${size} bytes`);
console.log(`imports (what it needs from the host): ${JSON.stringify(WebAssembly.Module.imports(module))}`);
console.log(`exports: ${WebAssembly.Module.exports(module).map((e) => e.name).join(", ")}\n`);

console.log("1. Calling exported functions");
console.log(`    add(2, 40)         = ${exports.add(2, 40)}`);
console.log(`    fibonacci(90)      = ${exports.fibonacci(90)}`); // u64 arrives as a BigInt
let start = performance.now();
const primes = exports.count_primes(2_000_000);
console.log(`    count_primes(2M)   = ${primes} (${(performance.now() - start).toFixed(1)} ms)`);

console.log("\n2. Passing text through shared memory");
const text = "Hello from JavaScript, running Rust with no operating system!";
const encoded = new TextEncoder().encode(text);
const memory = new Uint8Array(exports.memory.buffer);
const address = exports.buffer_address();
memory.set(encoded, address); // JavaScript writes into the module's memory…
console.log(`    vowels: ${exports.count_vowels(encoded.length)}`);
exports.to_upper_in_place(encoded.length); // …Rust changes it in place…
const result = new TextDecoder().decode(memory.slice(address, address + encoded.length));
console.log(`    upper:  ${result}`); // …and JavaScript reads it back

// Checks: this is the test.
assert.equal(exports.add(2, 40), 42);
assert.equal(exports.fibonacci(90), 2880067194370816120n);
assert.equal(primes, 148933);
// Count the vowels independently in JavaScript, and compare with Rust's answer.
const expectedVowels = (text.match(/[aeiou]/gi) ?? []).length;
assert.equal(exports.count_vowels(encoded.length), expectedVowels);
assert.equal(result, text.toUpperCase());
assert.deepEqual(WebAssembly.Module.imports(module), []);
console.log("\nall checks passed");

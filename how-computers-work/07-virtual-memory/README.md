<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 7: Virtual memory

## The idea in one sentence

Each ordinary running program has its own **numbering system for memory**; the processor translates those numbers into locations in RAM and checks what the program is allowed to do there.

Start with the hotel story and the tiny 16-byte-page example. The real page sizes and history can wait until the room-and-cubby idea feels familiar.

## The story: a magic hotel

Welcome to the Memory Hotel! Every guest gets a private room list: room 0, room 1, room 2, and so on. But these are the numbers **on the guest's list**, not the numbers painted on the hotel's real doors.

Ada asks for “my room 2”. Ben asks for “my room 2” too. They haven't booked the same bedroom! The hotel manager's **plan book** says:

| Guest's request | Hotel's actual room |
|---|---|
| Ada's room 2 | room 7 |
| Ben's room 2 | room 12 |

A speedy **doorman** checks whose list is being used, looks up the actual room, and checks the key. Ada only needs to remember her own numbers.

This is a *magic* hotel: rooms next to each other on a guest's list can be far apart in the real building. That is useful in a computer too: neighbouring virtual pages don't have to occupy neighbouring places in RAM.

| In the hotel | In the computer |
|---|---|
| one guest | one **process**: a running instance of a program |
| the guest's private room list | the process's **virtual address space** |
| a room on that list | a **virtual page** |
| an actual hotel room | a **physical page**, also called a **page frame**, in RAM |
| a numbered storage cubby inside a room | one **byte** inside a page |
| the manager | the **operating system** (OS) |
| the plan book | the **page tables**: mappings and permissions |
| the doorman | the **MMU**, the processor's memory management unit |
| the doorman's quick reminder cards | the **TLB**, a cache of translations |

A room is a whole page, **not one byte**. Let's open the plan book and see how that works.

## 1. Your room list belongs to you

**The story.** Ada's “room 2” means whatever Ada's list says. Using the same number from Ben's list doesn't open Ben's door. Some numbers on a list may not have a room assigned at all.

**In the computer.** A process uses **virtual addresses**. The same virtual address in two processes can lead to different physical memory. An address alone doesn't identify the process: the processor also needs the current process's mappings.

That is why the addresses printed in [lesson 2](../02-memory-and-addresses/) and [lesson 5](../05-real-machine-code/) are virtual addresses. They don't tell us which RAM chip holds the bytes. An address space also has gaps: a possible address is not automatically an address the process may use. [Microsoft's explanation of virtual address spaces](https://learn.microsoft.com/en-us/windows/win32/memory/virtual-address-space) describes this separation.

A **program** is the code you can start; a **process** is a running instance of it. Each start creates a process; this lesson also starts child processes for its experiments. One process can have several **threads**: separate paths through the instructions that share its address space. Threads can take turns on one CPU core or run in parallel on different cores. All the threads of one process share its room list.

**Can guests share?** Yes, if the manager arranges it. Different processes can share physical pages, for example read-only copies of the same code, or memory deliberately set up for exchanging data. Private address spaces provide protection; they don't require a separate physical copy of every byte. [Microsoft's memory-protection guide](https://learn.microsoft.com/en-us/windows/win32/memory/memory-protection) explains one way pages can be shared.

## 2. Rooms have cubbies: pages have bytes

**The story.** Each room contains numbered cubbies. “Room 2, cubby 5” tells the doorman both which room to find and where to look inside it. Moving to a different actual room doesn't change the cubby number.

**In the computer.** The system maps memory in blocks called **pages**. Common base page sizes are:

| System | Base page size |
|---|---|
| a native macOS process on the Apple M2 used for this course | **16 KiB** = 16,384 bytes |
| typical Linux or Windows PCs with Intel/AMD processors | **4 KiB** = 4,096 bytes |

**KiB** means 1,024 bytes. The OS and processor determine the available page sizes; some mappings use larger pages. The program queries the OS rather than guessing. You can also run `pagesize` on macOS or `getconf PAGESIZE` on Linux. [Apple recommends querying the page size](https://developer.apple.com/documentation/uikit/optimizing-memory-performance); [Microsoft describes pages and physical storage](https://learn.microsoft.com/en-us/windows/win32/memory/virtual-address-space-and-physical-storage).

A virtual address splits into a **page number** and an **offset**, the byte's position inside that page. The mapping replaces the page number; the offset stays the same.

For easy arithmetic, pretend our tiny hotel has **16-byte pages**. These are toy numbers, not this Mac's real page size:

```text
virtual address 37 = virtual page 2 × 16 + offset 5

Ada's map: virtual page 2 → physical page 7
physical address  = 7 × 16 + 5 = 117

Ben's map: virtual page 2 → physical page 12
physical address  = 12 × 16 + 5 = 197
```

Same virtual address, different physical addresses, same position inside each page!

**Follow Ada's request, one step at a time:**

| Step | Hotel action | Computer action |
|---|---|---|
| 1 | Ada asks for room 2, cubby 5. | Her process accesses virtual address 37. |
| 2 | The doorman uses Ada's list, not Ben's. | The MMU uses the current address-space mappings. |
| 3 | The list says room 7, and the key allows reading. | The translation identifies physical page 7 and permits the read. |
| 4 | The doorman opens cubby 5 in room 7. | The byte's physical address is 7 × 16 + 5 = **117**. |

This simple example assumes the page is ready in RAM. The doorman's reminder cards can speed up step 3; an unready or forbidden page needs the manager's attention, as the next sections explain.

The OS maintains page tables for the mappings. Here is a simplified view; all physical page numbers are invented:

| Part of a process's virtual memory | Physical backing | Typical permission |
|---|---|---|
| the lowest page, containing address 0 | normally unmapped | no access |
| a page of ordinary compiled code | physical page 7 | read and execute |
| a page containing string literals | physical page 12 | read only |
| a page of stack or heap data | physical page 20 | read and write |
| an allowed page that hasn't been filled yet | no private RAM page ready yet | the OS must prepare it before use |

Real page tables are usually organised in several levels, rather than one enormous list. The OS also keeps records of allowed memory regions, including pages that aren't ready in RAM yet. [The Linux page-table documentation](https://docs.kernel.org/mm/page_tables.html) explains the structures behind this picture.

## 3. The doorman remembers recent visits

**The story.** Looking through the whole plan book for every visit would be slow. So the doorman keeps quick reminder cards: “Ada's room 2 → actual room 7; looking allowed, changing allowed.” A reminder includes the rules, not just the room number.

**In the computer.** The **MMU** translates a process's memory accesses and checks page permissions. The **TLB** (*translation lookaside buffer*) caches translations and permissions, so many accesses don't need a fresh page-table lookup. It remembers *where* a page is; the data caches in [lesson 6](../06-the-cache/) remember *the bytes themselves*.

If a translation isn't cached, the processor needs to obtain it from the page tables. That takes extra work. A TLB miss alone doesn't mean a page is missing from RAM and doesn't necessarily cause a page fault. [Linux's MMU and TLB explanation](https://docs.kernel.org/mm/page_tables.html#mmu-tlb-and-page-faults) describes the difference.

## 4. A locked door, or a room that isn't ready?

**The story.** The doorman can't open a requested door and calls the manager. There are two very different possibilities:

- “That room is on your booking, but we need to prepare it. Please wait.”
- “Your key doesn't allow this. You can't do that.”

**In the computer.** A **page fault** is the processor handing a memory-access problem to the OS. It doesn't automatically mean a crash. The OS may prepare a valid page and let the instruction try again. If the access is forbidden, it normally reports an error that ends the process unless the process handles it.

Part 2 of the program makes deliberately invalid accesses. So that the crash doesn't end the whole lesson, the program starts a fresh copy of itself for each experiment, called a **child process**, and the child makes the bad access. The original program, the **parent**, waits for the child to finish and then reports its **exit status**: how it ended. Here is example output observed on macOS:

```text
reading address 16:            stopped by the operating system: signal 11, SIGSEGV (segmentation fault)
writing to a string constant:  stopped by the operating system: signal 10, SIGBUS (bus error)
```

On Unix-like systems such as Linux and macOS, the operating system tells a process about such a problem with a **signal**: a numbered message with a name. **SIGSEGV** (*segmentation violation*) means "invalid memory access"; **SIGBUS** (*bus error*) is a similar memory error, which macOS reports for this write. Unless the process has arranged to handle the signal, the OS stops it.

- **Reading address 16:** this is near address 0, in the lowest virtual page. Desktop systems normally leave that page unmapped to catch null-pointer mistakes. A **null pointer** itself has address **0**; address 16 is not null.
- **Writing to a string literal:** its page is normally read-only. Being allowed to read a page doesn't mean being allowed to change it.

Linux commonly reports `SIGSEGV` for both examples; Windows commonly reports an access violation. The exact result depends on the system and build.

**A Rust detail:** both child tasks deliberately break Rust's memory-access rules. That is **undefined behaviour**, meaning Rust promises no particular outcome, including no guaranteed crash. For a **valid** `read_volatile` or `write_volatile`, the access is an observable event that the compiler must preserve. But volatile operations still have memory-access requirements. These deliberately invalid operations break those requirements, so the guarantee doesn't apply. Neither volatile nor `unsafe` makes the access valid or guarantees a crash. These are demonstrations of bugs, not patterns to copy. The [Rust read documentation](https://doc.rust-lang.org/std/ptr/fn.read_volatile.html) and [write documentation](https://doc.rust-lang.org/std/ptr/fn.write_volatile.html) state the requirements.

**What the locks protect.** Page protection helps contain ordinary memory-access failures within a process. But it works at page boundaries: a wrong write into another value on an already writable page can pass the hardware checks. It doesn't replace Rust's checks, and it isn't a promise that no bug or attack can affect other programs.

**Can a room be a workshop?** Running bytes as instructions needs **execute** permission. Ordinary stack and heap pages are normally not executable. **W^X**, pronounced “write xor execute”, is a policy that prevents a mapping from being writable and executable at the same time. Systems enforce this with different rules and exceptions; special facilities support programs that generate machine code while running. This is the real-world protection related to [lesson 4](../04-a-tiny-cpu/)'s changeable recipe cards. [OpenBSD's W^X introduction](https://www.openbsd.org/33.html) describes the policy.

## 5. A booking isn't a furnished room

**The story.** Ada says, “I might need 50 rooms.” The manager records the booking without preparing all 50 rooms immediately. When Ada first needs one, the doorman calls the manager, who finds a room and prepares it. Then Ada's visit continues.

**In the computer.** This is **demand paging**: physical memory can be supplied when needed. For newly supplied **anonymous memory** (memory not filled from a file), the OS provides zeroed contents so the process doesn't see a previous user's data. A first read may use a shared zero-filled page; a first write may require a private writable page. Some pages may already be ready before your code touches them. [Apple's virtual-memory guide](https://developer.apple.com/library/archive/documentation/Performance/Conceptual/ManagingMemory/Articles/AboutMemory.html) describes deferred work and page faults.

Part 3 of the program times two operations:

```rust
// Excerpt: `size` is the buffer length chosen earlier in the program.
let mut memory = vec![0u8; size];  // request a buffer whose bytes read as zero
// Then write one byte in each base page covered by the buffer.
```

The default size is **1 GiB** = 1,073,741,824 bytes = 1,024 MiB. This is different from **1 GB** = 1,000,000,000 bytes.

A large zero-filled allocation can be quick when the **allocator** (the part of the program that hands out heap memory) and the OS put off preparing its physical pages. The later writes may take much longer because they require writable RAM pages. Another allocator or OS may do more work during allocation, so measure your own machine.

One run on the M2 Mac printed:

```text
OS base page size: 16384 bytes
asking for 1024 MiB of zeroed memory: 9.08µs
then writing one byte per base page: 204.55ms
base pages written: 65536 (this is not a page-fault count)
```

Here the booking was quick, and preparing writable rooms took much longer. Your timings may differ.

The program prints the queried base page size and the number of base pages it wrote to. It handles a buffer that starts or ends partway through a page. It **does not count page faults**: allocation may have touched some pages already, and larger mappings may cover several base pages. Dividing the elapsed time by the page count would not measure the exact cost of a page fault.

A page-aligned 1 GiB region contains **65,536 pages of 16 KiB**, or **262,144 pages of 4 KiB**. A buffer of that size that starts partway through a page spans one extra base page.

**Bookings still have limits.** An address space can be larger than available RAM. Some systems also allow **overcommit**, accepting requests without guaranteeing enough backing storage for every future write. Allocations can fail, and running out of memory later can cause processes to be ended. Virtual memory doesn't create unlimited storage. [Linux's overcommit documentation](https://kernel.org/doc/html/latest/mm/overcommit-accounting.html) explains these policies.

## 6. The hotel's storage basement

**The story.** The hotel needs more ready rooms. The manager packs away the contents of a little-used room in the basement, then reuses the actual room. Ada's room list stays the same. When Ada comes back, the manager restores the contents into a free room, updates the plan book, and lets her in.

**In the computer.** The OS can move page contents from RAM to a **swap file or swap area** on storage, then bring them back when needed. The virtual address can stay the same even if the page returns to a different physical location. Waiting for storage is much slower than using RAM.

Swap isn't always available. Systems may also compress little-used pages in RAM, or simply drop pages that are unchanged copies of a file, such as a program's code, because they can be read from the file again. If too little memory can be reclaimed, an allocation can fail or the OS may end a process.

**What if everyone keeps visiting the basement?** If useful pages keep being removed and immediately needed again, the computer spends too much time moving pages and too little time doing useful work. That slowdown is called **thrashing**. In hotel terms: the manager spends the whole day carrying suitcases up and down the stairs.

## 7. Change the room list for a new stay

**The story.** Suppose a thief has learned that Ada always keeps her treasure in “room 20” on her private list. At the next check-in, the manager assigns less predictable numbers to her bedroom and luggage store. An old guess is less useful.

This changes the **numbers on Ada's list**. Merely choosing different actual hotel rooms behind unchanged list numbers wouldn't hide the addresses her program uses.

**In the computer.** **Address space layout randomisation (ASLR)** makes the virtual locations of memory regions less predictable. Part 1 starts this executable three times and prints four addresses. Here is an example from one run on the M2 Mac:

```text
run 1: code 0x104b1d0cc  static 0x104b60728  stack 0x16b2e5fbf  heap 0x1052f5ba0
run 2: code 0x1029c90cc  static 0x102a0c728  stack 0x16d439fbf  heap 0x102fb1ba0
run 3: code 0x10076d0cc  static 0x1007b0728  stack 0x16f695fbf  heap 0x100fe9ba0
```

| Label | What the program prints |
|---|---|
| `code` | the address of the `child` function |
| `static` | the address of `COUNTER`, a `static` value that exists for the whole run |
| `stack` | the address of a local byte |
| `heap` | the address of a byte in a `Box` |

With ASLR enabled and a compatible executable, addresses often differ across fresh runs. Exactly which regions move depends on the OS, executable and settings. Addresses can repeat. ASLR makes some attacks harder; an attacker may still learn an address through another bug. [Apple explains ASLR's purpose and requirements](https://developer.apple.com/library/archive/documentation/Security/Conceptual/SecureCodingGuide/Articles/BufferOverflows.html).

**Spot the pattern.** The last three hexadecimal digits match in this example. If a region moves by a whole number of pages and an object keeps the same position within it, its offset within a page stays the same: the lowest **12 bits** for 4 KiB pages, or **14 bits** for 16 KiB pages. Stack and heap objects need not keep that position across runs, so “the last 14 bits of every address always stay the same” is not a rule. [The PaX ASLR design](https://pax.grsecurity.net/docs/aslr.txt) describes separate randomisation for different regions.

## Words to remember

| Word | Meaning |
|---|---|
| **process** | one running instance of a program: a hotel guest |
| **virtual address** | the byte address a process uses: its room number plus cubby number |
| **physical address** | the byte's location in physical memory |
| **page / page frame** | a virtual memory block / a physical RAM block: a listed room / an actual room |
| **page table** | mappings from virtual pages to physical pages, with permissions and status |
| **MMU** | processor hardware for address translation and page protection |
| **TLB** | a cache of translations and permissions |
| **page fault** | a memory-access problem handed to the OS; it may be recoverable |
| **segmentation fault** | an invalid-memory-access signal, normally ending the process on Unix-like systems |
| **demand paging** | preparing or bringing in pages when needed |
| **swapping** | moving page contents between RAM and swap storage |
| **thrashing** | spending too much time moving needed pages in and out |
| **ASLR** | randomising virtual memory layouts to make addresses harder to predict |

## A bit of history: who invented the magic hotel?

This extra timeline follows the same problem through the years: who should decide which pages stay ready?

<details>
<summary>Explore the hotel's history: Atlas, working sets and stronger locks</summary>

### Before paging: programmers carried the suitcases

Many early computers had very little fast memory. Programmers divided large programs into **overlays**: pieces loaded into the same memory area at different times. Someone had to plan which pieces were needed together. The hotel guest had to do much of the room juggling!

### 1962: Atlas puts the manager in charge

The **Atlas** computer, developed by the University of Manchester and Ferranti under **Tom Kilburn**, was inaugurated on 7 December 1962. It combined fast **magnetic-core memory** (tiny magnetic rings, each holding one bit) with much bigger but slower **magnetic drums** (spinning cylinders, a little like an early hard disk), moving data between them in pages of **512 words**. Its **one-level store** let programs use one set of addresses while the system moved pages between the two stores. A word on Atlas was 48 bits, not a modern byte. [Manchester's Atlas history](https://curation.cs.manchester.ac.uk/computer50/www.computer50.org/kgill/atlas/atlas.html) records these details.

### 1968: keep the rooms people actually use

**Peter Denning's working-set model** described a process's working set as the pages it had used during a recent interval. It helped explain how to manage memory and avoid thrashing: keep useful pages ready instead of constantly packing them away. This is a model for managing demand, not a guarantee that a computer can never slow down. [Denning's 1968 paper](https://denninginstitute.com/pjd/PUBS/WSModel_1968.pdf) introduced the model.

### The 1970s and 1980s: the idea spreads

IBM announced virtual-memory support for **System/370** in **1972**. Intel's **80386**, introduced in **1985**, supported paging with 4 KiB pages. Hardware support made protected virtual memory available to more systems, but the operating system still had to use it; old PC software didn't automatically gain protection. [IBM's System/370 history](https://www.ibm.com/history/system-370) and [Linux's paging history](https://docs.kernel.org/mm/page_tables.html) describe these milestones.

### The 2000s: stronger keys and less predictable lists

Defences such as **ASLR** made useful addresses harder to guess. **Non-executable data pages** made it harder to run code injected into data, and OpenBSD **3.3**, released in **2003**, introduced **W^X** on supported architectures. These protections made attacks harder, rather than making memory bugs harmless. [OpenBSD's release notes](https://www.openbsd.org/33.html) describe its initial architecture coverage.

### 2018: Meltdown shows why the locks need checking

**Meltdown**, publicly disclosed in 2018, showed that affected processors could leak memory a program wasn't allowed to read. The processor's work done ahead of time (**speculative execution**, like the guessing in [lesson 4](../04-a-tiny-cpu/)) was thrown away when the access turned out to be forbidden, but it left traces in the cache ([lesson 6](../06-the-cache/)), and a program could measure those traces. The doorman said no, but too late: the room had already been glimpsed. [The researchers' explanation](https://meltdownattack.com/) describes the attack.

One defence, **kernel page-table isolation (KPTI)**, removes most of the **kernel**'s pages (the core of the operating system, [lesson 8](../08-user-space-and-kernel-space/)) from the plan book while ordinary programs run. When enabled, it can add work to switches into and out of the kernel. The cost depends on the hardware and workload; this history doesn't explain the M2 timings above. [Linux's KPTI documentation](https://docs.kernel.org/arch/x86/pti.html) explains the mechanism and overhead.

### Today

Virtual memory is standard on modern desktop and phone operating systems. Some small computers built into devices, such as a washing machine's controller (**embedded** computers), run without it. The enduring idea is the same: programs use their own addresses while the system manages where their pages live and what access is allowed.

</details>

## Run it

From this lesson's directory:

```bash
cargo run --release
cargo test
```

The program shows **address comparisons**, **deliberately invalid child-process accesses**, and **allocation versus page-write timings**. The parent normally continues after the child crashes. Timings and addresses are examples to investigate, not fixed answers your computer must reproduce.

The default buffer is **1 GiB**. To try a smaller experiment, change `let size = 1 << 30;` in [src/main.rs](src/main.rs) to `let size = 64 << 20;` for **64 MiB**. The output reports the chosen size.

**Try being the doorman:** in our toy 16-byte-page hotel, Ada's virtual page 3 maps to physical page 9. Where does virtual address 50 lead?

<details>
<summary>Open the plan book for the answer</summary>

50 = 3 × 16 + 2, so it means page 3, offset 2. Replace page 3 with physical page 9: 9 × 16 + 2 = **physical address 146**.

</details>

Previous: [Lesson 6: The cache](../06-the-cache/) · Next: [Lesson 8: User space and kernel space](../08-user-space-and-kernel-space/)

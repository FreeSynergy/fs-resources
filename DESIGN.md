# fs-resources – Design

`fs-resources` knows what we have, who has time, what we need and who offers it. This repo holds the **program core**, no GUI. Contract:
`resources.capnp` in `fs-interfaces` v0.20.0 (draft, provisional: no standard for a resource model was found and no frontends are analysed). Program docs: `fs-documentation/docs/en/programs/apps/resources/`.

## State

Version 0.1.0, being built. The State table is filled in as parts are finished.

## Decisions (the owner may change them; all provisional)

### Answers to the page's open questions
1. **Things and people's time are one program,** because they share planning; the research note suggests the same. They stay in separate modules and separate store objects.
2. **ERP:** availability here, with a plain stock number for material. No warehouses, batches or valuation. **Costs** are out until they are analysed.

### Objects
Store types `Resource`, `Reservation`, `Need`, `Offer`, `Capacity`, `Allocation`, `GroupShare`; `context=resources`; IRIs `https://<instance>/resources/<id>`. No AS2 type exists for them, so content is in `fs:` members (namespace `https://freesynergy.net/ns#`). Index fields for every filter. Nothing here is an event or a task: those are only referred to by IRI (ADR 0050).

### Reservations
State pattern, one transition function. A pure sweep (Specification) computes what is held over time, and every check goes through it:
* `requested` holds nothing. `confirmed` holds `quantity - done` for `[from, to)`.
* `material` is used up: a confirmed reservation holds from `from` until finished, with no end, because what is used on Monday is gone on Tuesday.
* `confirm`, `change`, `setQuantity` and accepting an offer run under one lock across read and write, so two cannot overbook together. Clones share the lock.
* `finish` on a thing adds to `done` and frees that many; it ends when `done = quantity`; more than is out is refused. On material it lowers the stock by `quantity`, the rest returns, it ends at once. Refused once it has ended.
* **Stock is derived, so finishing is one write.** A material's stored value is its base; `quantity` = base minus the `done` of its ended reservations. `finish` writes only the reservation, so a crash cannot lower the stock twice or not at all. `setQuantity` compares `expected` with the derived value and stores the base that gives the new one.
* `free` returns stretches of equal free quantity over the asked range, adjacent equal stretches merged.
* A `room` is a thing with its own kind; the rules are those of a thing.
* Delete is refused while a confirmed reservation exists.

### Capacity
The repo is built with the acting person's IRI (`actor`), and every `Capacities` method acts for that person.
`as_actor(iri)` gives the same repo for another person and shares its locks; a server uses it per caller, because separate repos on one store do not share locks.
* Another person's record answers exactly like an unknown id, including in `remove`, `answer` and `withdraw`.
* `propose` always returns `proposed`, whether the person exists or not.
* `answer` can be repeated. `withdraw` only by the proposer. Only `accepted` counts in `load`.
* `load` over a range: capacity is minutes per week, pro rata by overlap; allocations pro rata by overlap, integer arithmetic, rounding documented and tested.
* Capacity, allocations and shares are written with `UnloggedObjects`, **in a store of their own** that the host passes separately and never connects to the hub. The log of an ordinary write would carry them there. There is no undo for them, which is stated.
* The hub manifest of this program must not list these types in `writes`; provide a const list of writable types, with a test.
* **`groupLoad`:** the caller must be a member, otherwise it answers exactly like too few people. Only members who switched sharing on for this exact group and have a non-zero capacity in the week count. Port `GroupLookup` (`members(group)`, effective members); no port: `Unsupported`. Ranges must be whole weeks from `from`; others are refused. Each week is summed and rounded on its own to a fixed 300 minutes, so a long range cannot be used to read a rate. A week with fewer than 5 contributors makes the whole answer `hidden`, `people` 0, empty totals. These numbers are constants here, not parameters. **Stated limits:** a one-minute change at a rounding boundary moves the total by a whole step, and someone leaving shows up within a step. An attacker who fills a group with their own accounts, who can agree to share, still gets close to one person's hours; a person has to agree to share with that group, and the interface says so.
* The RPC server must refuse `Capacities` until the transport knows the caller (contract header). Told to thread 3.

### Needs and offers
State pattern for both. `covered` = sum of accepted offers, and the need becomes `covered` when that reaches `quantity`, and goes back to `open` when it falls below. Offers on a closed need are refused. Accepting an offer that names a resource, on a need with a time, makes a confirmed reservation in the same lock and stores its id; if that is not free, nothing changes. Withdrawing or declining an accepted offer cancels its reservation. Such a reservation cannot be cancelled directly, and a need's time cannot change while an offer is accepted. The offer is saved before its reservation changes; a repeated `decide` settles what a crash left unfinished. A need's holder for that reservation is its `subject`.

### Feed
`subscribe` over the store's change feed: resources, reservations, needs, offers. Capacity types never arrive, because they are unlogged; tested.

### Not built
Costs, a time ledger, MSPDI import and export (fit unproven), skills, and scenarios. They have no contract method.

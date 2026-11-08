# FreeSynergy Resources — know what we have and need

**Two people cannot book the same room by accident, and nobody else can take your time.**

FreeSynergy Resources keeps what a group has and what it needs: things, rooms, material in stock, and people's time. You can reserve something for a stretch of time. You can post a need, and someone can post an offer.

What is different here: your own capacity, meaning your time, belongs to you. Only you decide what is shown and allocated, and what a group sees is a share you give. A reservation is checked in one step, so two people cannot overbook together. Material that is used up is counted as used up. Things and people's time are one program on purpose, because they share planning.

## What it can do today

Latest tag: `v0.2.0`.

- Things, rooms and material, with stock that is worked out from what has been finished.
- Reservations that cannot overbook; finishing a reservation frees the thing or uses up the material.
- Free time of a thing over a range.
- Your own capacity, allocations and shares to groups, kept in a store of its own.
- The load of a group, using the Groups interface.
- Needs and offers; accepting an offer makes the reservation.
- Tests: 115, with real stores. Checked by an independent reviewer over four rounds.

## What is not there yet

- No window and no server. A server must refuse your capacity calls until it knows who is asking.
- No costs, no MSPDI (project plan file) support, no skills.
- No warehouses, batches or valuation; this is availability, not an ERP.
- No standard for a resource model was found, so the interface is a draft marked provisional.

## Part of FreeSynergy

Resources belongs to the project-management and democracy suites. FreeSynergy is a family of programs that are free for everyone, forever, and that keep your data yours. The documentation, the manifesto and the overall plan live in [fs-documentation](https://github.com/FreeSynergy/fs-documentation); the reading version is at <https://freesynergy.github.io/fs-documentation/>. Start with the [manifesto](https://freesynergy.github.io/fs-documentation/manifesto/) if you want to know why we do this.


## For developers

- Workspace crate: `fs-resources-core` (core, no GUI).
- The contract is `resources.capnp` in [fs-interfaces](https://github.com/FreeSynergy/fs-interfaces) (draft, provisional). Resources owns it.
- Build and test: `cargo build --workspace` and `cargo test --workspace`.
- Design and decisions: [DESIGN.md](DESIGN.md).
- Licence: MIT OR Apache-2.0.

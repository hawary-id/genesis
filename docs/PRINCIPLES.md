# Architectural Principles

## Simulation First

Simulation depth > graphics.

---

## Emergence Over Features

Jika sesuatu bisa muncul secara alami, jangan hardcode.

---

## Systems Over Objects

Bangun interaksi antar sistem.

Hindari fitur yang terisolasi.

---

## Data-Oriented Architecture

Gunakan ECS untuk skalabilitas jangka panjang.

---

## Pressure-Driven Development

Layer baru hanya boleh dibangun jika layer sebelumnya menciptakan kebutuhan terhadap layer tersebut.

Contoh:

Scarcity -> Trade

Trade -> Specialization

Specialization -> Economy

Economy -> Institutions

Institutions -> Civilization

---

## Genesis Is Not A Weather Simulator

Environmental realism is useful only when it creates meaningful pressure for emergence.

Genesis should model terrain, climate, seasons, water, nutrients, minerals, and energy availability deeply enough to constrain future life, but not so deeply that Phase 1 becomes a meteorology or geology project.

Prefer simple deterministic environmental rules that generate long-term pressure over highly realistic short-term weather behavior.

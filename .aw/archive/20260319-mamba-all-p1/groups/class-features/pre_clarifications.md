---
change: mamba-all-p1
group: class-features
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: FULLY IMPLEMENTED. Descriptor protocol in class.rs:347-429. is_descriptor() and is_data_descriptor() check __get__/__set__/__delete__. 3-level lookup: data descriptors > instance dict > non-data descriptors. @property, @classmethod, @staticmethod handled as special descriptors.

### Q2: General
- **Answer**: C3 MRO IMPLEMENTED in class.rs:618-705 via compute_mro() + c3_merge(). super() works with explicit args (mb_super at class.rs:1165-1235). MISSING: zero-argument super() with compiler-injected __class__ cell.

### Q3: General
- **Answer**: PARTIALLY IMPLEMENTED. abc_mod.rs exports abc.ABC/abstractmethod/ABCMeta. class.rs:1086-1162 has ABSTRACT_METHODS registry, mb_abstractmethod() decorator, mb_check_abstract() validation. Limitation: ABCMeta is a stub dict, not a true metaclass.

### Q4: General
- **Answer**: IMPLEMENTED with validation. SLOTS_REGISTRY in class.rs:38-40. mb_register_slots() at 518-537. Validation in mb_setattr() at 494-511. Object model uses dynamic dict per instance (Instance { fields: RwLock<HashMap> } in rc.rs:65-68). Slots work as validation constraint on dynamic dict, not fixed-offset fields.


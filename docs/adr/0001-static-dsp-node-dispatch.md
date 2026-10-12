# Static Enum Dispatch for Realtime DSP Nodes

To satisfy real-time audio constraints requiring zero heap allocations (`no malloc`), all DSP modules implement a uniform seam through a statically sized tagged enum (`DspNode`) rather than dynamic heap trait objects (`Box<dyn DspNode>`). This guarantees fixed memory layout inside preallocated `DspSlot` storage, optimal instruction cache locality, and zero heap allocation during audio callback execution.

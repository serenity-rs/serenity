# Global data

This example demonstrates how to share data between events.

Global data, also referred to as user data, is accessible across all events and anywhere else that
has a copy of the Client's `data` Arc, e.g., inside commands via Context.

> **Tip:** For a demonstration of using the event handler struct to share data between events, see
[example 12] and [example 13].

[example 12]: /examples/e12_parallel_loops/
[example 13]: /examples/e13_sqlite_database/

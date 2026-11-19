# Collectors

This example showcases the beauty of collectors.

Collectors make it possible to await messages or reactions from a user in the middle of a control
flow, e.g., inside a command. They can also be used to await modal and component interactions, or
other events.

In this example, you will use the `CollectMessages` and `CollectReactions` traits, as well as the
fundamental `collect()` function that the specific collector types wrap.

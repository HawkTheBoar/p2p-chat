#set page(width: 8.5in, height: 11in)
= P2P Communication application 
created by Michal Stransky

with supervision from Ing. Zdenek Drvota

Institution: Delta skola SSIE
Documented on date: 2026-01-11



== Abstract Summary

The chat application is supposed to work in a mainly decentralised manner, meaning that every peer should communicate directly with each other

The application is aimed to solve the privacy concern of sending messages that could be read by the providers of centralised chat applications and holding metadata for example when you are communicating with whom

The main technologies used to build the application:
- Rust
- LibP2P
- Tokio (asynchronous runtime)
- Ratatui (user interface)
- Sqlite (local storage)
- mDNS (local peer discovery)

Key features:
- Configurable TUI with vim-like controls
- something
- something


== Introduction


== System requirements & constraints

Currently the application is only implemented for UNIX-like systems

== Background
=== Existing chat systems with similar purpose
Matrix

=== Protocols
- QUIC
- Noise
- mDNS

== Application Design

== Implementation
== Features & Functionality
== Security Considerations
== Results, Discussion & Limitations
Challenges that need to be faced:
- Storing messages for peers that will not come online for a long time on the DHT
- System for handling names of peers (deriving the hash for the DHT Node ID?) or by using Trackers
- etc.
== Conclusion & Future Work
=== Future work
- voice chat
- video calls
- hopefully a mobile version
- implement a GUI for the desktop

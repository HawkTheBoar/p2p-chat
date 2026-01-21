// #set page(width: 8.5in, height: 11in)
= P2P Komunikační aplikace
vytvořil Michal Stránský

pod vedením Ing. Zdeňka Drvoty

Instituce: Delta škola SŠIE
Zdokumentováno dne: 2026-01-11



== Abstraktní Shrnutí

Chatovací aplikace má fungovat převážně decentralizovaně, což znamená, že každý peer by měl v ideálním případě komunikovat napřímo s tím s kým chce momentálně komunikovat.

Aplikace má za cíl řešit problém ochrany soukromí při odesílání zpráv, které by mohly být čteny poskytovateli centralizovaných chatovacích aplikací, a uchovávání metadat, například kdy komunikujete s kým.

Hlavní technologie použité k vytvoření aplikace:
- Rust
- LibP2P
- Tokio (asynchronní runtime)
- Ratatui (uživatelské rozhraní)
- Sqlite (lokální úložiště)
- mDNS (lokální vyhledávání peerů)
- Noise (šifrování komunikace)
- QUIC (hlavní transport protokol)

Klíčové vlastnosti:
- Konfigurovatelné TUI s ovládacími prvky podobným jako ve vimu
- něco
- něco


== Úvod


== Systémové požadavky a omezení

V současné době je aplikace určena pouze pro systémy typu podporující POSIX.

== Pozadí
=== Stávající chatovací systémy s podobným účelem, ale jiným zpracováním
==== Matrix
- open-source
- každý může hostovat server který se zapojuje do decentralizovaného systému serverů, uživatelé komunikují pomocí těchto serverů //TODO
==== Keet by HolePunch 
- není open-source
- nemá implementaci persistant message storage přes DHT, tudíž přenos zpráv může proběhnout pouze když jsou oba uživatelé přístupní
=== Protokoly
==== QUIC
==== Noise
==== Multicast DNS
mDNS, neboli multicast Domain Name System, je způsob, jakým uzly používají IP multicast k publikování a přijímání DNS záznamů RFC 6762 v rámci lokální sítě.
mDNS se běžně používá v domácích sítích, aby se zařízení jako počítače, tiskárny a chytré televize mohly navzájem objevit a připojit.@libp2p-mdns

Aby mDNS discovery mohl fungovat MUSÍ uzel odesílat své mDNS dotazy z
   portu UDP 5353 a MUSÍ
   naslouchat na odpovědi mDNS odeslané na port UDP 5353 na
   adrese mDNS link-local multicast (224.0.0.251 a/nebo její IPv6
   ekvivalent FF02::FB).@mdnsrfc

== Návrh aplikace

== Implementace
== Vlastnosti a funkce
== Bezpečnostní aspekty
== Výsledky, diskuse a omezení
Výzvy, kterým je třeba čelit:
- Ukládání zpráv pro peer, kteří se dlouho nepřipojí k DHT
- Systém pro zpracování jmen peerů (odvození hash pro DHT Node ID?) nebo pomocí trackerů
- atd.
== Závěr a budoucí práce
=== Budoucí práce
- hlasový chat

#bibliography("ref.bib")

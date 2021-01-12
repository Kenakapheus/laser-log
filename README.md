# Laser Log

Ein kleines Tool um die Nutzung des Lasercutter zu Protokollieren.

Der Laser wird durch eine RFID Karte freigeschaltet, der Daemon kann die Eingabemaske mittels eines HTTP Request freischalten und wieder sperren.


---
## API

 - `POST /login/unlock`: Maske Freischalten
   - Parameter: `username` Benutzername der im Log hinterlegt werden soll
 - `POST /login/lock`: Maske Sperren
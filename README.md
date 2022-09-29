# EnProFMI2022
Entwicklungsprojekt am FMI 2022.
Im weiteren folgt eine Kurze Anleitung um Trailscout zu verwenden.
Weitergehende Fragen werden im [Wiki](https://github.tik.uni-stuttgart.de/st149207/EnProFMI2022/wiki) oder in der Ausarbeitung behandelt.

## Installation über Docker

Hierfür muss Docker installiert werden:
* Windows: https://docs.docker.com/desktop/windows/install/
* Linux: https://docs.docker.com/engine/install/#server (select your installed Linux distribution)

### Local Build 

Ist Docker installiert kann das Projekt in dev modus gestartet werden über: 

``docker compose up``

Durch diesen Befehl werden bestehende Docker image benutzt. Falls keine existieren werden diese automatisch gebaut. 
Die images können explizit gebaut werden mit (nötig um die genutzten images zu aktualisieren):

``docker compose build``

Das Bauen der images kann einige Minuten dauern.
Wurde das compose-file gestartet ist das frontend über den Port 80 erreichbar und das backend über den Port 8080.

Es lassen sich die images auch einzeln starten, hierzu sollte man sich aber an das Docker Wiki wenden.

### Production Build auf Linux

Download Repository:

`git clone https://github.tik.uni-stuttgart.de/st149207/EnProFMI2022.git`

**Die URL über die das Frontend erreicht werden soll muss in `gui/src/environments/environment.prod.ts` eingetragen werden.
Per Default ist die für Produktion auf Seewalze eingestellt!**

Um das Projekt in Production Environment auszuführen:

`sudo docker compose -f docker-compose.yml -f production.yml up`

bzw. um es auch neu zu bauen `sudo docker compose -f docker-compose.yml -f production.yml up --build`


## Graph Erstellung

Alle Befehle werden aus dem "backend" Verzeichnis ausgeführt.
Um das osmium preprocessing tool auszuführen mit source file für Bremen und output im Docker:
```
python3 preprocess_osm.py osm_graphs/bremen31-8-22.osm.pbf osm_graphs/bremen-compact.osm.pbf
```
Um den Graph Creator auszuführen mit source file für z.B. Bremen:
```
export i=./osm_graphs/bremen-compact.osm.pbf
export o=./osm_graphs/bremen-compact.fmi.bin
cargo run --bin osm_graph_creator
```
Input File Parameter, danach Output File Parameter.
Beim Umbenennen darauf achten was in der Server Conifg steht.

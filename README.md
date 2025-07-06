# lighthousectl

A command-line tool to control the power state of Valve Base Stations 2.0.

## Usage

### Scan All Base Stations

Continuously scans for all base stations. Press Ctrl-C to stop.

```console
$ lighthousectl scan
```

### Turn On All Base Stations

Continuously scans and powers on any discovered base stations. Press Ctrl-C to stop.

```console
$ lighthousectl on
```

### Scan Specific Base Stations

Scans until all specified base stations are found, then exits.

```console
$ lighthousectl scan LHB-01234567 LHB-89ABCDEF
```

### Turn On Specific Base Stations

Scans and powers on the specified base stations. Exits after all are turned on.

```console
$ lighthousectl on LHB-01234567 LHB-89ABCDEF
```

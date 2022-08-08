# lighthousectl

A command line tool to control the power state of Valve Base Stations 2.0.

## Usage

### Scan All Base Stations

It scans endlessly. You can stop by Ctrl-C.

```console
$ lighthousectl scan
```

### Turn On All Base Stations

It scans endlessly and turns on the all discovered base stations. You can stop by Ctrl-C.

```console
$ lighthousectl on
```

### Show Specified Base Stations

After the all specified base stations has been discovered, it exits.

```console
$ lighthousectl scan LHB-01234567 LHB-89ABCDEF
```

### Turn On Specified Base Stations

After the all specified base stations has been turned on, it exits.

```console
$ lighthousectl on LHB-01234567 LHB-89ABCDEF
```

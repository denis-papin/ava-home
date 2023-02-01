"# ava-home"

Control your Zigbee home devices



Family definition

```json
{
    "familyName" : "Lamp",
    "props" : {
        "state" : "BOOLEAN",
        "brightness" :  "INTEGER",
        "color" :  "RGB"
    }
}
```
Device declaration

```json
{
  "deviceName": "KitchenLamp",
  "family": "Lamp",
  "address": "0xahe5bad",
  "friendlyName": "kitchen_lamp"
}
```
Pattern, formerly a loop, here is the KitchenLoop
```json
{
  "patternName": "SwitchDoubleLamp",
  "devices": [
    {
      "family": "Switch",
      "address": "0x11111"
    },
    {
      "family": "Lamp",
      "address": "0xahe5bad"
    },
    {
      "family": "Lamp",
      "address": "0x22222"
    }
  ]
}
```
Above, SwitchDoubleLamp will be a pattern, ie a list of DynDevices, created dynamically 


```json
{
  "patternName": "GenericLightSwitch",
  "switches": [
    {
      "family": "Switch",
      "address": "0x11111"
    }
  ],
  "lamps": [
    {
      "family": "Lamp",
      "address": "0xahe5bad"
    }
  ]
}
```

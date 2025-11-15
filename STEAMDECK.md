# Steam Deck Support for godot-vlc

## Problem

El plugin godot-vlc puede no funcionar correctamente en Steam Deck debido a conflictos entre las bibliotecas VLC empaquetadas y el Steam Runtime. Los síntomas incluyen:

- El juego no inicia o crashea al cargar VLC
- Errores de bibliotecas no encontradas
- Problemas con aceleración de hardware
- Conflictos con libcurl u otras bibliotecas del sistema

## Solución

Hay dos enfoques para hacer funcionar godot-vlc en Steam Deck:

### Opción 1: Usar VLC del Sistema (Recomendado)

Esta es la solución más confiable para Steam Deck.

#### Paso 1: Instalar VLC en Steam Deck

1. Cambia a modo escritorio en Steam Deck (Menú Steam > Power > Switch to Desktop)

2. Abre Konsole (terminal) y ejecuta:

```bash
# Desactivar modo de solo lectura temporalmente
sudo steamos-readonly disable

# Actualizar base de datos de paquetes
sudo pacman -Sy

# Instalar VLC
sudo pacman -S vlc

# Reactivar modo de solo lectura (opcional pero recomendado)
sudo steamos-readonly enable
```

#### Paso 2: Configurar el Juego para Usar VLC del Sistema

En tu proyecto de Godot, agrega esta configuración antes de usar el plugin:

**Opción A: Modificar las dependencias del .gdextension**

Edita el archivo `addons/godot-vlc/godot_vlc.gdextension` y comenta las dependencias de Linux:

```ini
[dependencies]
# Comentar o eliminar las dependencias empaquetadas para Linux
# linux.x86_64 = {
#     "res://addons/godot-vlc/bin/linux-x64/libvlc.so.12": "",
#     "res://addons/godot-vlc/bin/linux-x64/libvlccore.so.9": "",
#     "res://addons/godot-vlc/bin/linux-x64/libidn.so.11": "",
#     "res://addons/godot-vlc/bin/linux-x64/vlc": ""
# }
```

**Opción B: Usar variables de entorno**

Antes de lanzar tu juego en Steam Deck, agrega al comando de lanzamiento:

```bash
LD_LIBRARY_PATH=/usr/lib %command%
```

Esto hará que el juego use las bibliotecas VLC del sistema en lugar de las empaquetadas.

#### Paso 3: Configurar las Opciones de Lanzamiento en Steam

1. En Steam (modo Gaming), ve a tu juego
2. Presiona el botón de opciones (☰)
3. Selecciona "Properties" > "General" > "Launch Options"
4. Agrega:

```bash
LD_LIBRARY_PATH=/usr/lib:/usr/lib64 %command%
```

### Opción 2: Usar Flatpak VLC

Si prefieres no modificar el sistema:

1. Instala VLC desde Discover (App Store de Steam Deck):
   - Cambia a modo escritorio
   - Abre Discover
   - Busca "VLC" e instala la versión Flatpak

2. Crea un script de lanzamiento que use las bibliotecas de Flatpak:

```bash
#!/bin/bash
export LD_LIBRARY_PATH=/var/lib/flatpak/runtime/org.freedesktop.Platform/x86_64/23.08/active/files/lib:$LD_LIBRARY_PATH
export VLC_PLUGIN_PATH=/var/lib/flatpak/app/org.videolan.VLC/x86_64/stable/active/files/lib/vlc/plugins
./tu_juego.x86_64
```

### Opción 3: Redistribuir con Bibliotecas Actualizadas

Para desarrolladores que quieren distribuir su juego con VLC incluido:

1. Recompila las bibliotecas VLC en un entorno compatible con Steam Deck (Arch Linux)
2. Reemplaza las bibliotecas en `addons/godot-vlc/bin/linux-x64/`
3. Asegúrate de incluir todas las dependencias necesarias

## Verificación

Para verificar que VLC funciona correctamente en Steam Deck:

1. Ejecuta tu juego en Steam Deck
2. Verifica en los logs de Godot que no hay errores relacionados con VLC
3. Prueba reproducir un video para confirmar que funciona

## Notas Importantes

- **Actualizaciones de SteamOS**: Las actualizaciones de SteamOS pueden revertir los cambios en el sistema. Es posible que necesites reinstalar VLC después de actualizaciones importantes.

- **Modo de Solo Lectura**: SteamOS usa un sistema de archivos de solo lectura para proteger el sistema. Desactívalo temporalmente con `sudo steamos-readonly disable` y reactívalo después con `sudo steamos-readonly enable`.

- **Aceleración de Hardware**: VLC del sistema generalmente tiene mejor soporte para aceleración de hardware en Steam Deck que las versiones empaquetadas.

- **Proton/Wine**: Si estás usando la versión Windows del juego a través de Proton, puede que tengas menos problemas, pero el rendimiento puede ser inferior.

## Troubleshooting

### Error: "libvlc.so.12: cannot open shared object file"

**Solución**: VLC no está instalado en el sistema. Sigue el Paso 1 de la Opción 1.

### Error: "version 'CURL_OPENSSL_4' not found"

**Solución**: Conflicto con libcurl de Steam. Usa:

```bash
LD_PRELOAD=/usr/lib/libcurl.so.4 %command%
```

### El video se reproduce pero con lag/stuttering

**Solución**: Problemas de aceleración de hardware. Intenta:

1. Usar VLC del sistema (Opción 1)
2. Configurar VLC para usar aceleración de hardware:
   - En Project Settings > VLC > Arguments, agrega: `--avcodec-hw=any`

### El audio no funciona

**Solución**: Verifica la configuración de audio de Godot y Steam Deck. Asegúrate de que el bus de audio esté configurado correctamente.

## Soporte Adicional

Si continúas teniendo problemas:

1. Revisa los logs del juego en Steam Deck
2. Verifica que VLC funcione correctamente en modo escritorio
3. Prueba con un video simple primero antes de videos complejos
4. Reporta issues en el repositorio de GitHub con:
   - Versión de SteamOS
   - Versión de VLC instalada
   - Logs del error
   - Configuración de lanzamiento usada

---

**Última actualización**: 2025-01-15

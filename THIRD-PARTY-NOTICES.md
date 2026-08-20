# Componentes de terceiros

O pacote do Lumina inclui executáveis necessários para processar mídia sem depender da configuração da máquina:

- FFmpeg e FFprobe 8.1.1 Essentials Build, distribuídos conforme a licença presente em `tools/FFMPEG-LICENSE.txt`. Informações do build estão em `tools/FFMPEG-README.txt`.
- ExifTool 13.59 e seu runtime, distribuídos conforme `tools/exiftool_files/LICENSE` e os avisos adicionais dentro dessa pasta.

Fixtures usadas apenas no código-fonte e nos testes:

- HEIC do projeto `dsoprea/heic-exif-samples`, licença MIT.
- DNG do raw.pixls.us, CC0/domínio público.

O aplicativo invoca essas ferramentas como processos separados e preserva os avisos e licenças correspondentes no pacote.

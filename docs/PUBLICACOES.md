# Onde ficam os arquivos .jwpub · Where .jwpub files go

**[Português](#português) · [English](#english)**

---

## Português

### Resumo

1. O mais fácil: **Biblioteca → aba BAIXAR**. Escolha as publicações e o idioma, vá em
   **Minha seleção** e toque em **Baixar agora**. Elas vão direto para a Biblioteca, e o app avisa
   quando sai edição nova ou atualização.
2. Ou baixe o `.jwpub` no jw.org (na página da publicação, formato **JWPUB**) e importe o arquivo.
   Há três jeitos, explicados abaixo.
3. Pronto: o app guarda uma cópia própria, e o arquivo baixado pode ser apagado.

Você **não precisa** colocar os `.jwpub` numa pasta especial. O app só lê o que foi importado.

### Os três jeitos de importar

| Jeito | Como |
|---|---|
| **Botão** | Início → **Importar publicações**, ou o ícone ⬇ no alto da Biblioteca. Dá para escolher vários arquivos de uma vez. |
| **Arrastar** | Arraste um ou mais `.jwpub` do gerenciador de arquivos para a janela do app. |
| **Pasta Downloads** | Se houver um `.jwpub` novo na sua pasta **Downloads**, aparece um aviso no Início com o botão **Importar**. Esse aviso pode ser desligado em Configurações. |

### Onde o app guarda as publicações

Ao importar, o app abre o `.jwpub` e guarda o conteúdo em:

```
~/.local/share/io.github.leitorjw/
├── biblioteca/
│   ├── nwtsty_T/          ← uma pasta por publicação (nome do arquivo sem ".jwpub")
│   │   ├── publicacao.db  ← o texto
│   │   ├── midia/         ← imagens
│   │   └── instalado.json
│   ├── w_T_202612/
│   └── mwb_T_202611/
├── busca.db               ← índice da busca na Bíblia (refeito sozinho se for apagado)
└── userData.db            ← seus grifos, notas, etiquetas e marcadores
```

`~` é a sua pasta pessoal. A pasta `.local` fica oculta: no gerenciador de arquivos, aperte
**Ctrl+H** para ver. O caminho é o mesmo no Flatpak e no AppImage.

### Bom saber

- **Não renomeie os arquivos.** O nome do arquivo vira o nome da pasta (`w_T_202612.jwpub` →
  `w_T_202612`). Um arquivo renomeado aparece como publicação repetida.
- **Importar de novo é seguro.** Se o arquivo é o mesmo, nada muda. Se é uma versão nova
  (tamanho diferente), o app substitui a antiga. Seus grifos e notas ficam, porque estão no
  `userData.db`, separados das publicações.
- **Bíblia:** a busca e as notas de estudo usam a *Tradução do Novo Mundo (Edição de Estudo)*,
  arquivo `nwtsty_<idioma>.jwpub` (em português: `nwtsty_T.jwpub`).
- **Reuniões:** a aba mostra a apostila e o estudo de *A Sentinela* da semana. Para isso, importe a
  apostila do bimestre (`mwb_…`) e a *Sentinela* de estudo do mês (`w_…`).
- **Qualquer idioma:** o app abre publicações em qualquer idioma. A letra no fim do nome indica o
  idioma (`T` = português, `E` = inglês, `S` = espanhol…).
- **Tirar uma publicação:** feche o app e apague a pasta dela em `biblioteca/`.
- **Mudar de computador:** copie a pasta `~/.local/share/io.github.leitorjw/` inteira. Para levar
  só grifos e notas, use **Estudo Pessoal → Backup**: o arquivo `.jwlibrary` também abre no
  JW Library do celular.

---

## English

### In short

1. The easiest: **Library → DOWNLOAD tab**. Pick publications and a language, go to **My
   selection** and tap **Download now**. They go straight into the Library, and the app tells you
   when a new issue or an update comes out.
2. Or download the `.jwpub` from jw.org (on the publication's page, **JWPUB** format) and import it.
   There are three ways, explained below.
3. Done: the app keeps its own copy, so you can delete the downloaded file.

You **don't need** to put `.jwpub` files in any special folder. The app only reads what you imported.

### Three ways to import

| Way | How |
|---|---|
| **Button** | Home → **Import publications**, or the ⬇ icon at the top of the Library. You can pick several files at once. |
| **Drag and drop** | Drag one or more `.jwpub` files from your file manager onto the app window. |
| **Downloads folder** | When there is a new `.jwpub` in your **Downloads** folder, a notice with an **Import** button shows up on Home. You can turn it off in Settings. |

### Where the app keeps publications

On import, the app unpacks the `.jwpub` into:

```
~/.local/share/io.github.leitorjw/
├── biblioteca/            ← "library"
│   ├── nwtsty_E/          ← one folder per publication (file name without ".jwpub")
│   │   ├── publicacao.db  ← the text
│   │   ├── midia/         ← images
│   │   └── instalado.json
│   └── w_E_202612/
├── busca.db               ← Bible search index (rebuilt automatically if deleted)
└── userData.db            ← your highlights, notes, tags and bookmarks
```

`~` is your home folder. `.local` is hidden: press **Ctrl+H** in your file manager to see it.
The path is the same for the Flatpak and the AppImage.

### Good to know

- **Don't rename the files.** The file name becomes the folder name (`w_E_202612.jwpub` →
  `w_E_202612`). A renamed file shows up as a duplicate publication.
- **Importing again is safe.** The same file changes nothing. A newer version (different size)
  replaces the old one. Your highlights and notes stay, because they live in `userData.db`,
  apart from the publications.
- **Bible:** search and study notes use the *New World Translation (Study Edition)*, file
  `nwtsty_<language>.jwpub` (English: `nwtsty_E.jwpub`).
- **Meetings:** the tab shows the week's meeting workbook and *Watchtower* study. Import the
  workbook (`mwb_…`) and the study *Watchtower* (`w_…`) for that period.
- **Any language:** the app opens publications in any language. The letter at the end of the name is
  the language (`E` = English, `T` = Portuguese, `S` = Spanish…).
- **Removing a publication:** close the app and delete its folder inside `biblioteca/`.
- **Moving to another computer:** copy the whole `~/.local/share/io.github.leitorjw/` folder. To
  move only highlights and notes, use **Personal Study → Backup**: the `.jwlibrary` file also opens
  in JW Library on your phone.

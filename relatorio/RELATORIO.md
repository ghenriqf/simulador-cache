# Relatório — Simulador de Cache e Experimentos com Estratégias de Organização

**UNIVALI — Organização de Computadores** · Prof. Thiago Felski Pereira, M.Sc.

**Autores:** Gabriel Henrique, João Matheus, Luciano Tomasi

**Código-fonte:** https://github.com/ghenriqf/simulador-cache

> **Nota sobre as previsões:** as previsões (anexo em [`FOLHA_PREVISOES.md`](FOLHA_PREVISOES.md)) registram
> o raciocínio teórico de primeira aproximação. As divergências estão explicadas na Conciliação de cada experimento.

---

## Como os números foram obtidos

Todas as tabelas abaixo foram geradas pelo simulador (`src/main.rs`) com o script `experimentos.ps1`, que
roda cada configuração e grava a saída completa (por acesso + resumo) em `resultados/`. Os números de
acesso citados nas justificativas (ex.: "acesso 18") referem-se à primeira coluna dessas saídas.

```powershell
cargo build --release
powershell -ExecutionPolicy Bypass -File calibrar.ps1      # Parte 2: 4/4 idênticos
powershell -ExecutionPolicy Bypass -File experimentos.ps1  # Parte 3: gera resultados/
```

Uma configuração isolada pode ser rodada à mão, por exemplo:

```
target\release\simcache tracos\experimentos\E2_registros.txt -c 256 -b 16 -v 1 -p wb
```

## Parte 2 — Calibração

| Traço | Configuração | Resultado |
|---|---|---|
| C1_mapeamento_direto | 128 B, bloco 16 B, 1 via, WB | idêntico ao esperado |
| C2_associativa_LRU | 64 B, bloco 16 B, 2 vias, WB | idêntico ao esperado |
| C3_escrita | 64 B, bloco 16 B, 1 via, WB e WT | idêntico ao esperado |
| C4_classificacao | 64 B, bloco 16 B, 1 via, WB | idêntico ao esperado |

A comparação é feita linha a linha pelo `calibrar.ps1` contra os arquivos `*_esperado.txt`.

---

## E1 — Tamanho do bloco numa varredura

Traço `E1_varredura.txt` (128 palavras lidas uma vez), capacidade 256 B, 1 via, write-back.
Saídas: `resultados/E1_bloco{4,8,16,32,64}.txt`.

### Previsão

| Bloco (B) | 4 | 8 | 16 | 32 | 64 |
|---|---|---|---|---|---|
| Taxa prevista | 0% | 50% | 75% | 87,5% | 93,75% |

Cada palavra é lida uma única vez, em sequência. Só a 1ª palavra de cada bloco deve faltar, então a taxa é
(n−1)/n, com n palavras por bloco. Os bytes lidos devem ser sempre 512 B, o tamanho do vetor.

### Medição

| Bloco (B) | Acertos | Taxa de acerto | Compuls. | Conflito | Capac. | Bytes lidos |
|---|---|---|---|---|---|---|
| 4 | 0 | 0,00% | 128 | 0 | 0 | 512 |
| 8 | 64 | 50,00% | 64 | 0 | 0 | 512 |
| 16 | 96 | 75,00% | 32 | 0 | 0 | 512 |
| 32 | 112 | 87,50% | 16 | 0 | 0 | 512 |
| 64 | 120 | 93,75% | 8 | 0 | 0 | 512 |

### Justificativa

**1. Taxa de acerto em função das palavras por bloco.** Com *n* = B/4 palavras por bloco, a varredura
falta na primeira palavra de cada bloco e acerta nas *n − 1* seguintes:

  **taxa = (n − 1) / n = 1 − 4/B**

| B | n | (n−1)/n | medido |
|---|---|---|---|
| 4 | 1 | 0/1 = 0% | 0,00% |
| 8 | 2 | 1/2 = 50% | 50,00% |
| 16 | 4 | 3/4 = 75% | 75,00% |
| 32 | 8 | 7/8 = 87,5% | 87,50% |
| 64 | 16 | 15/16 = 93,75% | 93,75% |

Na saída com bloco 16 B (`E4_vias1.txt`, mesma configuração), o acesso 1 (`0x10000000`) falta e os acessos
2, 3 e 4 (`0x10000004`, `…08`, `…0C`) acertam no mesmo bloco; o acesso 5 (`0x10000010`) falta de novo
porque começa o bloco seguinte.

**2. Tipos de falta.** Só apareceram faltas **compulsórias**. O traço lê cada palavra uma única vez, em
ordem; cada bloco é usado apenas enquanto a varredura passa por ele e nunca é pedido de novo. Toda falta é,
portanto, o primeiro acesso àquele bloco. Os blocos expulsos (o vetor tem 512 B e a cache 256 B, então a
segunda metade expulsa a primeira) nunca voltam a ser acessados, por isso nenhuma falta de conflito ou
capacidade aparece.

**3. Bytes lidos.** Não mudaram: 512 B em todas as configurações, que é exatamente o tamanho do vetor.
Blocos maiores fazem menos buscas (128 × 4 B, 64 × 8 B, …, 8 × 64 B), mas cada byte trazido é usado. Neste
traço o bloco maior **não tem custo de tráfego**: só antecipa dados que seriam pedidos logo em seguida
(localidade espacial perfeita).

### Conciliação

A previsão bateu exatamente nas cinco configurações: a varredura só tem localidade espacial, e a fórmula
(n−1)/n descreve todo o comportamento.

---

## E2 — Tamanho do bloco com dados espalhados

Traço `E2_registros.txt` (16 registros de 72 B; lê só o 1º campo de cada um; 4 passadas = 64 acessos),
capacidade 256 B, 1 via, write-back. Saídas: `resultados/E2_bloco{4,8,16,32,64}.txt`.
Os registros são numerados de **R0 a R15**; o registro *k* começa em `0x10000000 + 72·k`.

### Previsão

| Bloco (B) | 4 | 8 | 16 | 32 | 64 |
|---|---|---|---|---|---|
| Taxa prevista | 75% | 75% | 75% | 0% | 0% |

O passo de 72 B é maior que qualquer bloco, então não há ganho espacial. Enquanto a cache tem pelo menos 16
linhas (B ≤ 16), os 16 registros cabem, e só a 1ª passada falta (48/64 = 75%). Com 8 ou 4 linhas não cabem,
e o laço falta em tudo.

### Medição

| Bloco (B) | Acertos | Taxa de acerto | Compuls. | Conflito | Capac. | Bytes lidos |
|---|---|---|---|---|---|---|
| 4 | 48 | 75,00% | 16 | 0 | 0 | 64 |
| 8 | 48 | 75,00% | 16 | 0 | 0 | 128 |
| 16 | 24 | 37,50% | 16 | 24 | 0 | 640 |
| 32 | 0 | 0,00% | 16 | 0 | 48 | 2048 |
| 64 | 0 | 0,00% | 16 | 0 | 48 | 4096 |

### Justificativa

**1. Comparação com o E1.** O que decide o efeito do bloco é a **localidade espacial** do traço, isto é,
a distância entre dados usados em sequência comparada com o tamanho do bloco:

- No E1 os acessos são consecutivos (passo de 4 B); um bloco maior traz palavras que serão usadas logo em
  seguida, então a taxa só sobe.
- No E2 o passo é de 72 B, maior que qualquer bloco testado. Cada bloco trazido tem **uma** palavra útil e o
  resto é lixo. O único reaproveitamento é **temporal** (as passadas 2, 3 e 4). Aumentar o bloco não traz
  nenhum acerto novo e ainda diminui o número de linhas da cache (256/B), até que os 16 registros deixam de
  caber. Por isso a taxa cai com o bloco.

**2. Registros que disputam o mesmo índice com bloco 16 B** (`resultados/E2_bloco16.txt`, 16 conjuntos):

| Índice | Registros | Acessos da 1ª passada | Evidência |
|---|---|---|---|
| 4 | R1 (`0x10000048`) e R8 (`0x10000240`) | 2 e 9 | acesso 9 expulsa `0x10000040`; acesso 18 expulsa `0x10000240` |
| 13 | R3 (`0x100000D8`) e R10 (`0x100002D0`) | 4 e 11 | acesso 11 expulsa `0x100000D0`; acesso 20 expulsa `0x100002D0` |
| 6 | R5 (`0x10000168`) e R12 (`0x10000360`) | 6 e 13 | acesso 13 expulsa `0x10000160`; acesso 22 expulsa `0x10000360` |
| 15 | R7 (`0x100001F8`) e R14 (`0x100003F0`) | 8 e 15 | acesso 15 expulsa `0x100001F0`; acesso 24 expulsa `0x100003F0` |

Na 2ª passada (acessos 17–32), os 8 registros desses pares faltam em sequência e se expulsam mutuamente
(acessos 18, 20, 22, 24, 25, 27, 29, 31), enquanto os outros 8 acertam (17, 19, 21, 23, 26, 28, 30, 32).
O mesmo padrão se repete nas passadas 3 e 4: 8 faltas por passada × 3 passadas = **24 conflitos**.
Os índices 5, 7, 12 e 14 ficam vazios durante todo o traço, então não falta espaço: falta espalhamento.

**3. Bytes trazidos por byte realmente lido.** O programa lê 64 palavras × 4 B = **256 B** (64 B
distintos: o primeiro campo dos 16 registros).

| Bloco | Bytes lidos da memória | ÷ 256 B lidos pelo programa | ÷ 64 B distintos |
|---|---|---|---|
| 4 | 64 | 0,25 | 1 |
| 8 | 128 | 0,5 | 2 |
| 16 | 640 | 2,5 | 10 |
| 32 | 2048 | 8 | 32 |
| 64 | 4096 | 16 | 64 |

Com 4 e 8 B cada dado útil é buscado uma única vez (o valor menor que 1 na terceira coluna se deve às 48
releituras que acertaram). Com 64 B, a memória entrega 64 bytes para cada byte útil.

**4. Tipos de falta em cada configuração.**

- **4 B e 8 B: só 16 compulsórias.** Cada registro ocupa um bloco diferente, a cache tem 64 (ou 32) linhas e
  os 16 índices são todos distintos. Com 8 B (`E2_bloco8.txt`): índices 0, 9, 18, 27, 4, 13, 22, 31, 8, 17,
  26, 3, 12, 21, 30, 7 (acessos 1–16). A partir do acesso 17 tudo acerta (17, 18, 19, 20, …).
- **16 B: aparecem 24 de conflito.** A cache tem exatamente 16 linhas para 16 blocos, então uma cache
  totalmente associativa guardaria todos. Mas, com 1 via, quatro pares caem no mesmo índice (item 2). Como a
  totalmente associativa acertaria, a falta é de **conflito**.
- **32 B: 48 de capacidade.** São 8 linhas para 16 blocos distintos (os registros continuam em blocos
  diferentes porque 72 > 32). Mesmo totalmente associativa, uma cache de 8 blocos percorrida em ciclo por
  16 blocos sob LRU sempre expulsa o próximo a ser usado e falta em tudo. Logo, todas as faltas após a 1ª
  passada são de **capacidade**. Exemplo: acesso 17 (`0x10000000`) falta e expulsa `0x10000300`, e o
  acesso 18 expulsa `0x10000240`.
- **64 B: 48 de capacidade.** O mesmo raciocínio vale com apenas 4 linhas. Já na 1ª passada o acesso 5
  (`0x10000120`) expulsa `0x10000000`.

### Conciliação

A previsão acertou 4, 8, 32 e 64 B, mas errou **16 B**: previmos 75% e o resultado foi 37,5%. O erro foi
olhar só o **número de linhas** (16 linhas para 16 registros, "cabe") e esquecer que, com 1 via, cada bloco
tem um único lugar possível, definido pelo índice. Como 72 não é múltiplo de 16, os registros não se
distribuem pelos 16 índices de forma uniforme: R1/R8, R3/R10, R5/R12 e R7/R14 caem no mesmo índice
(acessos 9, 11, 13 e 15), enquanto os índices 5, 7, 12 e 14 ficam vazios. Ter capacidade total suficiente
não garante ausência de faltas no mapeamento direto, e é exatamente isso que a classificação mostra:
são 24 faltas de **conflito**, não de capacidade.

---

## E3 — Associatividade contra conflito

Traço `E3_dois_vetores.txt` (A[i] + B[i], 32 palavras cada, B começa 256 B depois de A), capacidade 256 B,
bloco 16 B, write-back. Saídas: `resultados/E3_vias{1,2,4,16}.txt`.

### Previsão

| Vias | 1 | 2 | 4 | 16 |
|---|---|---|---|---|
| Taxa prevista | 0% | 75% | 75% | 75% |

A distância de 256 B põe A[i] e B[i] no mesmo índice, e com 1 via um expulsa o outro a cada acesso. Com 2 ou
mais vias os dois cabem no conjunto, e só a 1ª palavra de cada bloco falta.

### Medição

| Vias | Acertos | Taxa de acerto | Compuls. | Conflito | Capac. |
|---|---|---|---|---|---|
| 1 | 0 | 0,00% | 16 | 48 | 0 |
| 2 | 48 | 75,00% | 16 | 0 | 0 |
| 4 | 48 | 75,00% | 16 | 0 | 0 |
| 16 | 48 | 75,00% | 16 | 0 | 0 |

### Justificativa

**1. Mapeamento direto.** A distância entre A[i] e B[i] é de 256 B, exatamente a capacidade. Com 1 via e
16 conjuntos de 16 B, o índice usa os bits 4–7 do endereço, e somar 256 (bit 8) não altera esses bits.
Então A[i] e B[i] caem **sempre no mesmo índice**, com rótulos diferentes. Como o laço alterna A e B, um
expulsa o outro a cada acesso (`E3_vias1.txt`):

- acesso 1 `R 0x10000000` idx=0 → falta;
- acesso 2 `R 0x10000100` idx=0 → falta, expulsa `0x10000000`;
- acesso 3 `R 0x10000004` idx=0 → falta (**conflito**), expulsa `0x10000100`;
- acesso 4 `R 0x10000104` idx=0 → falta (**conflito**), expulsa `0x10000000`; e assim até o acesso 64.

Os dois vetores juntos ocupam 256 B (16 blocos) e caberiam numa cache totalmente associativa de 16 blocos.
Por isso as 48 faltas que não são compulsórias são todas de **conflito**: a cache não está cheia, só usa
mal os índices (os índices 0–7 recebem tudo e os 8–15 ficam vazios).

**2. Com 2, 4 e 16 vias.** Com 2 vias, cada conjunto guarda A[i] e B[i] ao mesmo tempo (`E3_vias2.txt`):
os acessos 1 e 2 faltam (compulsórias) e os acessos 3–8 acertam, porque as outras 3 palavras de cada bloco
já estão na cache. Sobram apenas as 16 faltas compulsórias (uma por bloco): 48/64 = 75%. Com 4 e 16 vias o
resultado é idêntico, porque depois de 2 vias não há mais conflito para eliminar e as faltas restantes são
todas compulsórias, que nenhuma associatividade remove. Duas vias bastam porque, a cada instante, só dois
blocos disputam cada conjunto.

**3. Traço modificado** (`tracos/experimentos/E3_dois_vetores_modificado.txt`): o segundo vetor passa a
começar em **`0x10000080`** (128 B depois de A, e não 256 B).

```
R 0x10000000
R 0x10000080
R 0x10000004
R 0x10000084
...
R 0x1000007C
R 0x100000FC
```

| Vias | Acertos | Taxa de acerto | Compuls. | Conflito | Capac. |
|---|---|---|---|---|---|
| 1 | 48 | 75,00% | 16 | 0 | 0 |

Por que funciona: A ocupa 128 B, que são os blocos de índice 0 a 7. Ao começar B em `+0x80`, B ocupa os
índices 8 a 15 (`E3mod_vias1.txt`: acesso 1 vai para idx=0, acesso 2 para idx=8, acesso 9 para idx=1,
acesso 10 para idx=9). Nenhum bloco de A divide índice com um de B, então nada é expulso e os acessos 3–8
acertam. 75% é o **máximo possível**: os 64 acessos tocam 16 blocos distintos, e as 16 faltas compulsórias
são inevitáveis (64 − 16 = 48 acertos). Qualquer início de B congruente a `0x80` módulo `0x100` (ex.:
`0x10000180`) dá o mesmo resultado.

### Conciliação

A previsão bateu nas quatro configurações. O único ponto a acrescentar é que 2 vias já bastam: com 4 e
16 vias não há ganho adicional, porque as 16 faltas restantes são compulsórias.

---

## E4 — Associatividade numa varredura

Traço `E1_varredura.txt`, capacidade 256 B, bloco 16 B, write-back. Saídas: `resultados/E4_vias{1,2,4,16}.txt`.

### Previsão

| Vias | 1 | 2 | 4 | 16 |
|---|---|---|---|---|
| Taxa prevista | 75% | 75% | 75% | 75% |

Não há reuso de blocos, só faltas compulsórias, e a associatividade não muda isso.

### Medição

| Vias | Acertos | Taxa de acerto | Compuls. | Conflito | Capac. |
|---|---|---|---|---|---|
| 1 | 96 | 75,00% | 32 | 0 | 0 |
| 2 | 96 | 75,00% | 32 | 0 | 0 |
| 4 | 96 | 75,00% | 32 | 0 | 0 |
| 16 | 96 | 75,00% | 32 | 0 | 0 |

### Justificativa

**1. Resultado pela classificação das faltas.** As 32 faltas são todas **compulsórias**, uma por bloco do
vetor (512 B / 16 B = 32 blocos). O traço nunca volta a um bloco já usado. Mesmo com 16 vias (totalmente
associativa, `E4_vias16.txt`), o acesso 1 falta e os acessos 2–4 acertam; o acesso 5 (`0x10000010`) falta
por ser o primeiro acesso a um bloco novo. Quando a cache enche, o acesso 65 (`0x10000100`) expulsa
`0x10000000`, mas esse bloco nunca mais é pedido, então a expulsão não gera falta nenhuma. Sem conflito e
sem capacidade, não há nada que a associatividade possa melhorar: a taxa fica em 75% (3 de cada 4 palavras)
em todas as configurações.

**2. O que a associatividade elimina.** Juntando E3 e E4: a associatividade elimina faltas de
**conflito** (no E3, 48 conflitos com 1 via viram 0 com 2 vias) e **não** elimina faltas
**compulsórias** (no E4 as 32 continuam com 16 vias). Também não elimina faltas de **capacidade**: no E5
(80 palavras), elas aumentam de 16 com 1 via para 40 com 16 vias. A totalmente associativa é justamente a
referência usada para definir o que é capacidade.

### Conciliação

A previsão bateu: 75% em todas as configurações, todas as faltas compulsórias.

---

## E5 — Laço maior e menor que a cache

Traços `E5_laco_80.txt` (80 palavras = 320 B = 20 blocos, 3 passadas, 240 acessos) e `E5_laco_60.txt`
(60 palavras = 240 B = 15 blocos, 3 passadas, 180 acessos). Capacidade 256 B (16 blocos), bloco 16 B,
write-back. Saídas: `resultados/E5_80_vias*.txt` e `resultados/E5_60_vias*.txt`.

### Previsão

| Vias | 1 | 2 | 4 | 16 |
|---|---|---|---|---|
| 80 palavras | 75% | 75% | 75% | 75% |
| 60 palavras | 91,67% | 91,67% | 91,67% | 91,67% |

80 palavras (320 B) não cabem em 256 B. Prevemos que o laço expulsa tudo a cada passada, sobrando só o ganho
espacial (3 de 4 palavras), e que mais vias, no máximo, empatariam. 60 palavras (15 blocos) cabem inteiras:
só as 15 compulsórias faltam (165/180).

### Medição — 80 palavras

| Vias | Acertos | Taxa de acerto | Compuls. | Conflito | Capac. |
|---|---|---|---|---|---|
| 1 | 204 | 85,00% | 20 | 0 | 16 |
| 2 | 196 | 81,67% | 20 | 0 | 24 |
| 4 | 180 | 75,00% | 20 | 0 | 40 |
| 16 | 180 | 75,00% | 20 | 0 | 40 |

### Medição — 60 palavras

| Vias | Acertos | Taxa de acerto | Compuls. | Conflito | Capac. |
|---|---|---|---|---|---|
| 1 | 165 | 91,67% | 15 | 0 | 0 |
| 2 | 165 | 91,67% | 15 | 0 | 0 |
| 4 | 165 | 91,67% | 15 | 0 | 0 |
| 16 | 165 | 91,67% | 15 | 0 | 0 |

### Justificativa

Chamando de b0…b19 os 20 blocos do vetor de 80 palavras (b*k* começa em `0x10000000 + 16·k`):

**1. Ordem: 1 via (85,00%) > 2 vias (81,67%) > 4 vias = 16 vias (75,00%).** A regra que explica tudo é
esta: num conjunto com *v* vias que recebe *k* blocos percorridos sempre na mesma ordem, se *k > v*, o LRU
sempre expulsa exatamente o bloco que será usado mais à frente, e **todo** acesso desse conjunto falta. Se
*k ≤ v*, todos ficam. Então o que importa é quantos conjuntos recebem mais blocos do que vias:

| Vias | Conjuntos | Distribuição dos 20 blocos | Blocos que sobrevivem entre passadas | Faltas por passada (2ª e 3ª) |
|---|---|---|---|---|
| 1 | 16 | conj. 0–3: 2 blocos; conj. 4–15: 1 bloco | 12 (b4…b15) | 8 |
| 2 | 8 | conj. 0–3: 3 blocos; conj. 4–7: 2 blocos | 8 (b4–b7, b12–b15) | 12 |
| 4 | 4 | todos os conj.: 5 blocos | 0 | 20 |
| 16 | 1 | 1 conj.: 20 blocos | 0 | 20 |

Faltas além das 20 compulsórias: 8×2 = 16, 12×2 = 24, 20×2 = 40 e 40, como na tabela. Mais vias juntam mais
blocos no mesmo conjunto e o LRU passa a "estragar" o conjunto inteiro, em vez de só uma parte. Com 2 vias
(`E5_80_vias2.txt`), o conjunto 0 recebe b0, b8 e b16 em ciclo: o acesso 81 (b0) expulsa b8 (`0x10000080`),
o 113 (b8) expulsa b16 (`0x10000100`) e o 145 (b16) expulsa b0. Cada um sai pouco antes de ser pedido de novo.

As faltas são classificadas como **capacidade** e não como conflito, porque a totalmente associativa de 16
blocos também falta em todo acesso (20 blocos em ciclo, mais do que ela comporta). Repare que, neste traço, o
mapeamento direto acerta **mais** do que a totalmente associativa: o LRU é a pior política possível para um
laço maior que a cache, e o mapeamento direto, por não poder escolher a vítima, acaba protegendo os blocos
b4–b15.

**2. Por que 1 via se saiu assim** (`E5_80_vias1.txt`). Na 1ª passada, os acessos 1, 5, …, 61 trazem b0…b15
para os índices 0–15. Os acessos 65, 69, 73 e 77 trazem b16…b19, que caem nos índices 0–3 e expulsam b0…b3
(`expulsa 0x10000000`, `…10`, `…20`, `…30`). Na 2ª passada:

- acessos 81, 85, 89, 93 (b0–b3) **faltam** e expulsam b16–b19;
- acessos 97 a 144 (b4–b15, índices 4–15) **acertam todos**: esses blocos sobreviveram desde a 1ª passada;
- acessos 145, 149, 153, 157 (b16–b19) **faltam** e expulsam b0–b3.

A 3ª passada repete o padrão (faltas em 161–173 e 225–237). Sobrevivem de uma passada para a outra os
**12 blocos b4…b15** (`0x10000040`…`0x100000F0`), e só os 4 pares de índices 0–3 trocam de lugar: 8 faltas
por passada.

**3. Traço de 60 palavras.** São 15 blocos, menos que as 16 linhas da cache. Com qualquer número de vias
todos cabem ao mesmo tempo: com 1 via ocupam os índices 0–14 sem repetir, e com 2, 4 e 16 vias nenhum
conjunto recebe mais blocos do que vias. Só existem as 15 faltas compulsórias da 1ª passada
(`E5_60_vias1.txt`: acessos 1, 5, …, 57), e as passadas 2 e 3 acertam tudo: 165/180 = 91,67%, que é o máximo
possível para 15 blocos.

**4. Mais vias ou conjunto de trabalho menor?** **Conjunto de trabalho menor**, de longe. Com 80 palavras,
aumentar as vias **piorou** a taxa (85% → 81,67% → 75%). Reduzindo o laço para 60 palavras, todas as
configurações vão para 91,67%, e as faltas nas passadas 2 e 3 caem de 16–40 para **zero**. Associatividade
só ajuda contra conflito; quando o problema é o conjunto de trabalho não caber na cache (capacidade),
a solução é fazê-lo caber.

### Conciliação

O traço de 60 palavras bateu. No de 80 palavras, a previsão acertou 4 e 16 vias (75%) e errou 1 via
(85%) e 2 vias (81,67%). Dois erros de raciocínio:

1. Supusemos que, se o vetor não cabe, **nada** sobrevive de uma passada para a outra. Isso só vale quando
   todos os blocos disputam o mesmo grupo de linhas, como na totalmente associativa. Com 1 via, só os
   índices 0–3 recebem dois blocos; os 12 blocos b4…b15 têm o índice só para si e sobrevivem (acertos dos
   acessos 97–144 na 2ª passada).
2. Supusemos que mais vias **nunca pioram**. Com LRU e acesso cíclico maior que o conjunto, a vítima é sempre
   o próximo bloco a ser usado. Quanto mais vias, mais blocos se juntam nos mesmos conjuntos e mais
   conjuntos passam a ter mais blocos do que vias: nenhum com 1 via além dos índices 0–3, os conjuntos 0–3
   com 2 vias, todos com 4 e 16 vias. A ordem 1 > 2 > 4 = 16 vem daí.

---

## E6 — Políticas de escrita

Capacidade 256 B, bloco 16 B, 2 vias. Saídas: `resultados/E6_{acumulador,copia,esparsa}_{wt,wb}.txt`.
Em WT, "escritas" são palavras de 4 B enviadas à memória. Em WB, são blocos de 16 B gravados (writebacks
durante a execução + descarga final).

### Previsão

| Traço | Acumulador | Cópia | Escritas esparsas |
|---|---|---|---|
| Menos bytes escritos | WB | WB | WB |

O write-back só grava um bloco quando ele sai da cache (ou no fim), enquanto o write-through grava a cada
escrita. Esperávamos que juntar as escritas sempre economizasse bytes. A taxa de acerto deveria ser igual
nas duas políticas, por causa do write-allocate.

### Medição — E6_acumulador (contador lido e reescrito 50 vezes)

| Política | Acertos | Taxa de acerto | Compuls. | Conflito | Capac. | Escritas | Bytes escritos | Bytes lidos |
|---|---|---|---|---|---|---|---|---|
| WT | 99 | 99,00% | 1 | 0 | 0 | 50 | 200 | 16 |
| WB | 99 | 99,00% | 1 | 0 | 0 | 1 (0 wb + 1 descarga) | 16 | 16 |

### Medição — E6_copia (cópia de 64 palavras)

| Política | Acertos | Taxa de acerto | Compuls. | Conflito | Capac. | Escritas | Bytes escritos | Bytes lidos |
|---|---|---|---|---|---|---|---|---|
| WT | 96 | 75,00% | 32 | 0 | 0 | 64 | 256 | 512 |
| WB | 96 | 75,00% | 32 | 0 | 0 | 16 (8 wb + 8 descarga) | 256 | 512 |

### Medição — E6_esparsa (32 escritas, cada uma num bloco diferente)

| Política | Acertos | Taxa de acerto | Compuls. | Conflito | Capac. | Escritas | Bytes escritos | Bytes lidos |
|---|---|---|---|---|---|---|---|---|
| WT | 0 | 0,00% | 32 | 0 | 0 | 32 | 128 | 512 |
| WB | 0 | 0,00% | 32 | 0 | 0 | 32 (16 wb + 16 descarga) | 512 | 512 |

### Justificativa

**1. A taxa de acerto não muda** entre WT e WB nos três traços: 99%, 75% e 0% nas duas políticas. Pelas
regras do simulador ela não tem como mudar. As duas políticas usam **write-allocate**, então uma escrita
que falta traz o bloco exatamente como uma leitura, e o LRU é atualizado do mesmo jeito. A política só
decide **o que é mandado para a memória** (a palavra na hora, ou o bloco quando sai); o conteúdo da cache e
a escolha das vítimas são idênticos. Comparando `E6_copia_wt.txt` e `E6_copia_wb.txt`, cada acesso tem o
mesmo resultado. A única diferença por acesso é a marca `(writeback)` em WB, por exemplo no acesso 66
(`W 0x10000480 … expulsa 0x10000400 (writeback)`).

**2. Escritas e bytes por traço.**

- **Acumulador:** WT envia 50 escritas × 4 B = 200 B. WB não expulsa o bloco do contador nenhuma vez
  (1 falta só) e grava o bloco uma vez, na descarga final: 1 × 16 B = 16 B. **WB escreve menos** (12,5× menos).
- **Cópia:** WT envia 64 × 4 B = 256 B. WB grava os 16 blocos do vetor destino, cada um uma vez: 8 por
  writeback durante a execução (ex.: acessos 66, 74, 82, quando a 2ª metade do destino expulsa a 1ª) e 8 na
  descarga final, ou 16 × 16 B = 256 B. **Empate em bytes**, mas WB faz 16 operações contra 64 do WT.
- **Esparsa:** WT envia 32 × 4 B = 128 B. WB precisa gravar cada um dos 32 blocos inteiros (16 expulsos
  com writeback, ex.: acesso 17 `expulsa 0x10000000 (writeback)`, e 16 na descarga final): 32 × 16 B =
  512 B. **WT escreve menos** (4× menos).

**3. Característica decisiva (uma frase):** vence a política conforme o **número de escritas que cada bloco
recebe enquanto está na cache**. WB paga 16 B por bloco sujo e WT paga 4 B por escrita, então com mais de
4 escritas por bloco vence WB (acumulador: 50), com exatamente 4 empata (cópia: 4 palavras por bloco) e com
menos de 4 vence WT (esparsa: 1).

**4. Escritas esparsas: bytes lidos × bytes escritos.** O programa escreveu 32 × 4 B = **128 B** e nunca
leu nada, mas a memória entregou **512 B** (32 blocos × 16 B), 4× o que foi escrito e 100% inútil. Isso
acontece por causa do **write-allocate**: cada escrita que falta (acessos 1–32, todas compulsórias) traz
o bloco inteiro de 16 B para dentro da cache só para alterar 4 B dele, e esse bloco nunca é lido. Sem
alocação (write-no-allocate), as escritas que faltam iriam direto para a memória sem trazer o bloco:
**0 B lidos**, os mesmos 0 acertos e 128 B escritos (32 palavras) nas duas políticas, já que nenhum bloco
ficaria sujo na cache. O tráfego total cairia de 640 B (WT) ou 1024 B (WB) para 128 B.

### Conciliação

A previsão acertou o acumulador e a igualdade das taxas de acerto, mas errou a cópia (empate em 256 B) e as
escritas esparsas (WT escreveu 128 B e WB 512 B). O erro foi contar só **quantas** operações cada política
faz e esquecer o **tamanho** de cada uma: o WB faz menos gravações, mas cada uma tem 16 B, contra 4 B do WT.
O WB só compensa quando um bloco recebe mais de 4 escritas enquanto está na cache. Na cópia cada bloco do
destino recebe exatamente 4 (empate em bytes, embora o WB faça 16 operações contra 64). Nas esparsas cada
bloco recebe só 1 escrita, e o WB grava 16 B para salvar 4 B úteis.

---

## Desafio — Projetem o traço

Arquivo: `tracos/desafio/desafio.txt` (40 acessos). Capacidade 256 B, bloco 16 B.
Saídas: `resultados/Desafio_vias{1,2,4}.txt`.

```
# repetido 10 vezes:
R 0x10000000
R 0x10000100
R 0x10000200
R 0x10000300
```

### Medição

| Vias | Acertos | Taxa de acerto | Compuls. | Conflito | Capac. |
|---|---|---|---|---|---|
| 1 | 0 | 0,00% | 4 | 36 | 0 |
| 2 | 0 | 0,00% | 4 | 36 | 0 |
| 4 | 36 | 90,00% | 4 | 0 | 0 |

Condições: 1 via = 2 vias = 0% (≤ 10%) ✔; 4 vias = 90% (≥ 70%) ✔.

### Como o traço foi construído

1. **Forçar todos os blocos para o mesmo conjunto em todas as configurações.** Com bloco de 16 B, o índice
   usa os bits 4–7 (1 via, 16 conjuntos), 4–6 (2 vias, 8 conjuntos) ou 4–5 (4 vias, 4 conjuntos). Endereços
   que diferem por múltiplos de 256 B (`0x100`) só diferem do bit 8 para cima, então caem no **conjunto 0
   em qualquer uma das três caches**: só o rótulo muda.
2. **Usar exatamente 4 blocos em rodízio fixo.** Pela regra do LRU vista no E5, um conjunto que recebe
   *k* blocos em ciclo falta em tudo se *k* > vias e acerta tudo (depois do 1º ciclo) se *k* ≤ vias.
   Com *k* = 4:
   - **1 via:** cada acesso expulsa o anterior e todos faltam.
   - **2 vias:** o conjunto guarda só os 2 últimos blocos, e o próximo pedido é sempre o mais antigo, que
     acabou de sair. `Desafio_vias2.txt`: acesso 3 (`0x10000200`) expulsa `0x10000000`, acesso 4 expulsa
     `0x10000100`, acesso 5 (`0x10000000`) falta e expulsa `0x10000200`, e assim até o fim: 0 acertos.
   - **4 vias:** os 4 blocos cabem no conjunto. `Desafio_vias4.txt`: acessos 1–4 faltam (compulsórias) e,
     do acesso 5 em diante, tudo acerta: 36/40 = 90%.
3. **Por que 2 vias não bastam e 4 resolvem:** dobrar as vias de 1 para 2 não muda nada porque 2 ainda é
   menor que 4 blocos disputando o conjunto; o LRU sempre descarta quem vai ser usado em seguida. Com 4 vias,
   a capacidade do conjunto alcança o conjunto de trabalho. As faltas de 1 e 2 vias são de **conflito**
   (a cache de 16 blocos guardaria os 4 com folga), e é por isso que a associatividade as elimina.
4. Repetir o ciclo 10 vezes dilui as 4 compulsórias: com *N* repetições, a taxa com 4 vias é
   (4N − 4)/4N. Bastam 4 repetições para passar de 70% (75%); com 10 temos margem (90%).

---

## Anexo — Folha de previsões

Ver [`FOLHA_PREVISOES.md`](FOLHA_PREVISOES.md).

# Anexo — Folha de previsões

**UNIVALI – Organização de Computadores** · Prof. Thiago Felski Pereira, M.Sc.

**Integrantes:** Gabriel Henrique, João Matheus, Luciano Tomasi

> Previsões baseadas no raciocínio teórico de primeira aproximação (localidade espacial, número de linhas
> da cache e "mais vias nunca piora"). As divergências são discutidas na seção **Conciliação** de cada
> experimento do relatório.

## E1 — varredura

| Bloco (B) | 4 | 8 | 16 | 32 | 64 |
|---|---|---|---|---|---|
| Taxa prevista | 0% | 50% | 75% | 87,5% | 93,75% |

**Por quê:** cada palavra é lida uma única vez, em sequência. Só a 1ª palavra de cada bloco falta, então a
taxa é (n−1)/n com n = palavras por bloco.

## E2 — registros

| Bloco (B) | 4 | 8 | 16 | 32 | 64 |
|---|---|---|---|---|---|
| Taxa prevista | 75% | 75% | 75% | 0% | 0% |

**Por quê:** o passo de 72 B é maior que qualquer bloco, então não há ganho espacial. Enquanto a cache tem
pelo menos 16 linhas (B ≤ 16), os 16 registros cabem e só a 1ª passada falta (48/64). Com 8 ou 4 linhas
não cabem, e a cache falta em tudo.

## E3 — dois vetores

| Vias | 1 | 2 | 4 | 16 |
|---|---|---|---|---|
| Taxa prevista | 0% | 75% | 75% | 75% |

**Por quê:** a distância de 256 B põe A[i] e B[i] no mesmo índice, e com 1 via um expulsa o outro sempre.
Com 2 ou mais vias os dois cabem no conjunto e só a 1ª palavra de cada bloco falta.

## E4 — varredura

| Vias | 1 | 2 | 4 | 16 |
|---|---|---|---|---|
| Taxa prevista | 75% | 75% | 75% | 75% |

**Por quê:** não há reuso de blocos, só faltas compulsórias, e a associatividade não muda isso.

## E5 — laços de 80 e de 60 palavras

| Vias | 1 | 2 | 4 | 16 |
|---|---|---|---|---|
| 80 palavras | 75% | 75% | 75% | 75% |
| 60 palavras | 91,67% | 91,67% | 91,67% | 91,67% |

**Por quê:** 80 palavras (320 B) não cabem em 256 B. Prevemos que o laço expulsa tudo a cada passada, sobrando
só o ganho espacial (3 de 4 palavras), com mais vias no máximo empatando. 60 palavras (15 blocos) cabem
inteiras: só as 15 compulsórias faltam (165/180).

## E6 — políticas de escrita: qual política escreve menos bytes na memória?

| Traço | Acumulador | Cópia | Escritas esparsas |
|---|---|---|---|
| WT, WB ou empate | WB | WB | WB |

**Por quê:** o write-back só grava um bloco quando ele sai da cache (ou no fim), enquanto o write-through grava
a cada escrita. Esperávamos que juntar as escritas sempre economizasse bytes.

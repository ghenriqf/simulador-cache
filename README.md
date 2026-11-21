# Simulador de Cache

Simulador de memória cache em Rust, feito para a avaliação prática de **Organização de Computadores (UNIVALI)**.
O enunciado completo está em [`docs/enunciado.pdf`](docs/enunciado.pdf).

**Autores:** Gabriel Henrique, João Matheus, Luciano Tomasi

## Funcionalidades

- Cache configurável: capacidade, tamanho do bloco e número de vias (de mapeamento direto até totalmente associativa)
- Substituição **LRU** dentro do conjunto
- Políticas de escrita **write-back** (bit de modificação, writeback na expulsão e descarga final) e **write-through**, ambas com *write-allocate*
- Classificação de cada falta em **compulsória**, **capacidade** ou **conflito**, usando uma cache totalmente associativa LRU simulada em paralelo
- Saída por acesso (rótulo, índice, offset, acerto/falta, bloco expulso, writeback) e resumo final
- Recusa configurações impossíveis (valores que não são potência de 2, mais vias do que blocos etc.)

## Requisitos

[Rust](https://rustup.rs) 1.85 ou mais recente (edição 2024). Não há dependências externas.

## Compilação

```sh
cargo build --release
```

O executável é gerado em `target/release/simcache` (`simcache.exe` no Windows).

## Uso

```
simcache <traco> -c <capacidade> -b <bloco> -v <vias|full> -p <wb|wt> [--resumo]
```

| Opção | Descrição |
|---|---|
| `-c`, `--capacidade` | bytes de dados da cache (potência de 2) |
| `-b`, `--bloco` | tamanho do bloco em bytes (potência de 2, ≥ 4) |
| `-v`, `--vias` | número de vias; `1` = mapeamento direto, `full` = totalmente associativa |
| `-p`, `--politica` | `wb` (write-back, padrão) ou `wt` (write-through) |
| `-r`, `--resumo` | mostra só o resumo final, sem a saída por acesso |

Exemplo:

```sh
simcache tracos/calibracao/C1_mapeamento_direto.txt -c 128 -b 16 -v 1 -p wb
```

```
CONFIGURACAO  capacidade=128 B  bloco=16 B  vias=1  conjuntos=8  escrita=WB
CAMPOS        rotulo=25 bits  indice=3 bits  offset=4 bits

   1  R 0x10000000  rot=0x0200000  idx=0   off=0   FALTA  compulsoria
   2  R 0x10000004  rot=0x0200000  idx=0   off=4   ACERTO
   ...
  10  R 0x100000FC  rot=0x0200001  idx=7   off=12  FALTA  compulsoria expulsa 0x10000070

acessos ............. 10
acertos ............. 3
faltas .............. 7  (compulsorias 5, conflito 2, capacidade 0)
taxa de acerto ...... 30.00%
blocos lidos da mem.  7  (112 B)
escritas na memoria  0  (0 B)  [writebacks 0 + descarga final 0]
```

### Formato do traço

Um acesso por linha: `R` (leitura) ou `W` (escrita), seguido do endereço em hexadecimal. Tudo depois de `#` é comentário.

```
# exemplo
R 0x10000000
W 0x10000004
```

## Calibração

O script `calibrar.ps1` roda os quatro traços de calibração e compara a saída, caractere por caractere, com os arquivos `*_esperado.txt`:

```powershell
powershell -ExecutionPolicy Bypass -File calibrar.ps1
```

```
OK     C1_mapeamento_direto
OK     C2_associativa_LRU
OK     C3_escrita
OK     C4_classificacao

Calibracao completa: 4/4 identicos.
```

## Estrutura

```
├── src/main.rs          código do simulador
├── tracos/
│   ├── calibracao/      traços C1–C4 e respectivas saídas esperadas
│   └── experimentos/    traços E1–E6
├── docs/enunciado.pdf   enunciado do trabalho
└── calibrar.ps1         verificação automática da calibração
```

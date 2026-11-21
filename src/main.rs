//! Simulador de cache — Organização de Computadores (UNIVALI)
//! Autores: Gabriel Henrique, João Matheus, Luciano Tomasi

use std::collections::HashSet;
use std::fmt::Write as _;
use std::process::ExitCode;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Politica {
    WriteBack,
    WriteThrough,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Op {
    Leitura,
    Escrita,
}

struct Config {
    capacidade: u32,
    bloco: u32,
    vias: u32,
    politica: Politica,
}

impl Config {
    fn blocos(&self) -> u32 {
        self.capacidade / self.bloco
    }

    fn conjuntos(&self) -> u32 {
        self.capacidade / (self.bloco * self.vias)
    }

    fn validar(&self) -> Result<(), String> {
        for (nome, v) in [
            ("capacidade", self.capacidade),
            ("bloco", self.bloco),
            ("vias", self.vias),
        ] {
            if v == 0 || !v.is_power_of_two() {
                return Err(format!("{nome}={v} não é potência de 2"));
            }
        }
        if self.bloco < 4 {
            return Err(format!("bloco={} B é menor que uma palavra (4 B)", self.bloco));
        }
        if self.bloco > self.capacidade {
            return Err(format!(
                "bloco={} B é maior que a capacidade={} B",
                self.bloco, self.capacidade
            ));
        }
        if self.vias > self.blocos() {
            return Err(format!(
                "vias={} é maior que o número de blocos ({})",
                self.vias,
                self.blocos()
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Linha {
    rotulo: u32,
    suja: bool,
}

/// Cada conjunto guarda só as linhas válidas, da mais recente (posição 0)
/// para a usada há mais tempo (última posição, a vítima do LRU).
struct Cache {
    conjuntos: Vec<Vec<Linha>>,
    vias: usize,
}

struct Resultado {
    acerto: bool,
    expulsa: Option<Linha>,
}

impl Cache {
    fn new(num_conjuntos: u32, vias: u32) -> Self {
        Cache {
            conjuntos: (0..num_conjuntos)
                .map(|_| Vec::with_capacity(vias as usize))
                .collect(),
            vias: vias as usize,
        }
    }

    fn acessar(&mut self, indice: u32, rotulo: u32, suja: bool) -> Resultado {
        let conj = &mut self.conjuntos[indice as usize];
        if let Some(pos) = conj.iter().position(|l| l.rotulo == rotulo) {
            let mut linha = conj.remove(pos);
            linha.suja |= suja;
            conj.insert(0, linha);
            return Resultado { acerto: true, expulsa: None };
        }
        let expulsa = if conj.len() == self.vias { conj.pop() } else { None };
        conj.insert(0, Linha { rotulo, suja });
        Resultado { acerto: false, expulsa }
    }

    fn linhas_sujas(&self) -> u64 {
        self.conjuntos.iter().flatten().filter(|l| l.suja).count() as u64
    }
}

#[derive(Default)]
struct Estatisticas {
    acessos: u64,
    acertos: u64,
    compulsorias: u64,
    conflito: u64,
    capacidade: u64,
    escritas_wt: u64,
    writebacks: u64,
    descarga_final: u64,
}

fn ler_traco(caminho: &str) -> Result<Vec<(Op, u32)>, String> {
    let texto = std::fs::read_to_string(caminho)
        .map_err(|e| format!("não foi possível ler '{caminho}': {e}"))?;
    let mut acessos = Vec::new();
    for (n, linha) in texto.trim_start_matches('\u{feff}').lines().enumerate() {
        let linha = linha.split('#').next().unwrap_or("").trim();
        if linha.is_empty() {
            continue;
        }
        let erro = |msg: &str| format!("{caminho}:{}: {msg}: '{linha}'", n + 1);
        let mut partes = linha.split_whitespace();
        let op = match partes.next().map(|s| s.to_ascii_uppercase()).as_deref() {
            Some("R") => Op::Leitura,
            Some("W") => Op::Escrita,
            _ => return Err(erro("operação inválida (esperado R ou W)")),
        };
        let end_txt = partes.next().ok_or_else(|| erro("endereço ausente"))?;
        let hex = end_txt
            .strip_prefix("0x")
            .or_else(|| end_txt.strip_prefix("0X"))
            .unwrap_or(end_txt);
        let end = u32::from_str_radix(hex, 16).map_err(|_| erro("endereço inválido"))?;
        if end % 4 != 0 {
            return Err(erro("endereço não alinhado a palavra de 4 bytes"));
        }
        if partes.next().is_some() {
            return Err(erro("texto extra na linha"));
        }
        acessos.push((op, end));
    }
    Ok(acessos)
}

fn simular(cfg: &Config, traco: &[(Op, u32)], resumo_apenas: bool) -> String {
    let bits_off = cfg.bloco.trailing_zeros();
    let bits_idx = cfg.conjuntos().trailing_zeros();
    let bits_rot = 32 - bits_off - bits_idx;
    let larg_rot = bits_rot.div_ceil(4).max(1) as usize;
    let wb = cfg.politica == Politica::WriteBack;

    let mut cache = Cache::new(cfg.conjuntos(), cfg.vias);
    // Totalmente associativa com a mesma capacidade: separa conflito de capacidade.
    let mut sombra = Cache::new(1, cfg.blocos());
    let mut vistos: HashSet<u32> = HashSet::new();
    let mut est = Estatisticas::default();

    let mut out = String::new();
    let _ = writeln!(
        out,
        "CONFIGURACAO  capacidade={} B  bloco={} B  vias={}  conjuntos={}  escrita={}",
        cfg.capacidade,
        cfg.bloco,
        cfg.vias,
        cfg.conjuntos(),
        if wb { "WB" } else { "WT" }
    );
    let _ = writeln!(
        out,
        "CAMPOS        rotulo={bits_rot} bits  indice={bits_idx} bits  offset={bits_off} bits"
    );
    out.push('\n');

    for (i, &(op, end)) in traco.iter().enumerate() {
        let num_bloco = end >> bits_off;
        let off = end & (cfg.bloco - 1);
        let idx = num_bloco & (cfg.conjuntos() - 1);
        let rot = num_bloco.checked_shr(bits_idx).unwrap_or(0);
        let escrita = op == Op::Escrita;

        est.acessos += 1;
        if escrita && !wb {
            est.escritas_wt += 1;
        }

        let r = cache.acessar(idx, rot, escrita && wb);
        let acerto_sombra = sombra.acessar(0, num_bloco, false).acerto;
        let primeira_vez = vistos.insert(num_bloco);

        let mut linha = format!(
            "{:>4}  {} 0x{:08X}  rot=0x{:0larg_rot$X}  idx={:<4}off={:<4}",
            i + 1,
            if escrita { 'W' } else { 'R' },
            end,
            rot,
            idx,
            off
        );

        if r.acerto {
            est.acertos += 1;
            linha.push_str("ACERTO");
        } else {
            let tipo = if primeira_vez {
                est.compulsorias += 1;
                "compulsoria"
            } else if !acerto_sombra {
                est.capacidade += 1;
                "capacidade"
            } else {
                est.conflito += 1;
                "conflito"
            };
            let _ = write!(linha, "FALTA  {tipo:<11}");
            if let Some(v) = r.expulsa {
                let end_v =
                    ((v.rotulo as u64) << (bits_idx + bits_off)) as u32 | (idx << bits_off);
                let _ = write!(linha, " expulsa 0x{end_v:08X}");
                if v.suja {
                    est.writebacks += 1;
                    linha.push_str(" (writeback)");
                }
            }
        }
        if !resumo_apenas {
            out.push_str(linha.trim_end());
            out.push('\n');
        }
    }
    est.descarga_final = cache.linhas_sujas();

    let faltas = est.acessos - est.acertos;
    let taxa = if est.acessos > 0 {
        100.0 * est.acertos as f64 / est.acessos as f64
    } else {
        0.0
    };
    let blk = cfg.bloco as u64;

    if !resumo_apenas {
        out.push('\n');
    }
    let _ = writeln!(out, "acessos ............. {}", est.acessos);
    let _ = writeln!(out, "acertos ............. {}", est.acertos);
    let _ = writeln!(
        out,
        "faltas .............. {}  (compulsorias {}, conflito {}, capacidade {})",
        faltas, est.compulsorias, est.conflito, est.capacidade
    );
    let _ = writeln!(out, "taxa de acerto ...... {taxa:.2}%");
    let _ = writeln!(out, "blocos lidos da mem.  {}  ({} B)", faltas, faltas * blk);
    if wb {
        let total = est.writebacks + est.descarga_final;
        let _ = writeln!(
            out,
            "escritas na memoria  {}  ({} B)  [writebacks {} + descarga final {}]",
            total,
            total * blk,
            est.writebacks,
            est.descarga_final
        );
    } else {
        let _ = writeln!(
            out,
            "escritas na memoria  {}  ({} B)",
            est.escritas_wt,
            est.escritas_wt * 4
        );
    }
    out
}

const USO: &str = "\
uso: simcache <traco> -c <capacidade> -b <bloco> -v <vias|full> -p <wb|wt> [--resumo]

  -c, --capacidade  bytes de dados da cache (potência de 2)
  -b, --bloco       tamanho do bloco em bytes (potência de 2, >= 4)
  -v, --vias        número de vias (1 = mapeamento direto; 'full' = totalmente associativa)
  -p, --politica    wb (write-back) ou wt (write-through); padrão wb
  -r, --resumo      mostra só o resumo final, sem a saída por acesso

exemplo: simcache tracos/calibracao/C1_mapeamento_direto.txt -c 128 -b 16 -v 1 -p wb";

fn parse_args() -> Result<(String, Config, bool), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        return Err(USO.to_string());
    }
    let mut traco = None;
    let (mut cap, mut blk, mut vias_txt) = (None, None, None);
    let mut politica = Politica::WriteBack;
    let mut resumo = false;

    let num = |s: &str, nome: &str| {
        s.parse::<u32>()
            .map_err(|_| format!("valor inválido para {nome}: '{s}'"))
    };

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        let mut valor = || {
            it.next()
                .ok_or_else(|| format!("faltou o valor de {a}\n\n{USO}"))
        };
        match a.as_str() {
            "-c" | "--capacidade" => cap = Some(num(&valor()?, "capacidade")?),
            "-b" | "--bloco" => blk = Some(num(&valor()?, "bloco")?),
            "-v" | "--vias" => vias_txt = Some(valor()?),
            "-p" | "--politica" => {
                politica = match valor()?.to_ascii_lowercase().as_str() {
                    "wb" | "write-back" => Politica::WriteBack,
                    "wt" | "write-through" => Politica::WriteThrough,
                    p => return Err(format!("política inválida: '{p}' (use wb ou wt)")),
                }
            }
            "-r" | "--resumo" => resumo = true,
            s if s.starts_with('-') => return Err(format!("opção desconhecida: {s}\n\n{USO}")),
            _ if traco.is_none() => traco = Some(a),
            _ => return Err(format!("argumento a mais: {a}\n\n{USO}")),
        }
    }

    let traco = traco.ok_or(format!("faltou o arquivo de traço\n\n{USO}"))?;
    let capacidade = cap.ok_or(format!("faltou -c <capacidade>\n\n{USO}"))?;
    let bloco = blk.ok_or(format!("faltou -b <bloco>\n\n{USO}"))?;
    let vias_txt = vias_txt.ok_or(format!("faltou -v <vias>\n\n{USO}"))?;
    let vias = if vias_txt.eq_ignore_ascii_case("full") {
        if bloco == 0 { 0 } else { capacidade / bloco }
    } else {
        num(&vias_txt, "vias")?
    };

    let cfg = Config { capacidade, bloco, vias, politica };
    cfg.validar().map_err(|e| format!("configuração impossível: {e}"))?;
    Ok((traco, cfg, resumo))
}

fn main() -> ExitCode {
    let (caminho, cfg, resumo) = match parse_args() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    match ler_traco(&caminho) {
        Ok(traco) => {
            print!("{}", simular(&cfg, &traco, resumo));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("erro: {e}");
            ExitCode::FAILURE
        }
    }
}

# Roda todos os experimentos (E1-E6 e Desafio) e salva a saida completa de cada
# configuracao em resultados\, mais uma tabela-resumo em resultados\tabelas.md.
# Uso: powershell -ExecutionPolicy Bypass -File experimentos.ps1
$ErrorActionPreference = 'Stop'
$raiz = $PSScriptRoot
$sim  = Join-Path $raiz 'target\release\simcache.exe'
$exp  = Join-Path $raiz 'tracos\experimentos'
$des  = Join-Path $raiz 'tracos\desafio'
$out  = Join-Path $raiz 'resultados'

if (-not (Test-Path $sim)) {
    Push-Location $raiz
    cargo build --release --quiet
    Pop-Location
}
New-Item -ItemType Directory -Force $out | Out-Null

function Get-Num([string]$texto, [string]$padrao) {
    if ($texto -match $padrao) { return $Matches[1] } else { return '' }
}

# Roda uma configuracao, salva a saida e devolve os numeros do resumo.
function Invoke-Exp([string]$nome, [string]$traco, [int]$cap, [int]$blk, [int]$vias, [string]$pol) {
    $saida = (& $sim $traco -c $cap -b $blk -v $vias -p $pol) -join "`r`n"
    $saida | Out-File (Join-Path $out "$nome.txt") -Encoding utf8
    [pscustomobject]@{
        Acertos = Get-Num $saida 'acertos \.+ (\d+)'
        Taxa    = (Get-Num $saida 'taxa de acerto \.+ ([\d\.]+%)')
        Comp    = Get-Num $saida 'compulsorias (\d+)'
        Conf    = Get-Num $saida 'conflito (\d+)'
        Cap     = Get-Num $saida 'capacidade (\d+)\)'
        Lidos   = Get-Num $saida 'blocos lidos da mem\.\s+\d+\s+\((\d+) B\)'
        Escr    = Get-Num $saida 'escritas na memoria\s+(\d+)'
        BEscr   = Get-Num $saida 'escritas na memoria\s+\d+\s+\((\d+) B\)'
    }
}

$md = New-Object System.Collections.Generic.List[string]

function Add-Tabela([string]$titulo, [string]$col, $linhas, [switch]$Lidos, [switch]$Escrita) {
    $md.Add("### $titulo"); $md.Add('')
    $cab = "| $col | Acertos | Taxa de acerto | Compuls. | Conflito | Capac. |"
    $sep = '|---|---|---|---|---|---|'
    if ($Escrita) { $cab += ' Escritas | Bytes escritos |'; $sep += '---|---|' }
    if ($Lidos -or $Escrita) { $cab += ' Bytes lidos |'; $sep += '---|' }
    $md.Add($cab); $md.Add($sep)
    foreach ($l in $linhas) {
        $r = $l.R
        $s = "| $($l.Chave) | $($r.Acertos) | $($r.Taxa) | $($r.Comp) | $($r.Conf) | $($r.Cap) |"
        if ($Escrita) { $s += " $($r.Escr) | $($r.BEscr) |" }
        if ($Lidos -or $Escrita) { $s += " $($r.Lidos) |" }
        $md.Add($s)
    }
    $md.Add('')
}

$md.Add('# Tabelas dos experimentos (geradas por experimentos.ps1)'); $md.Add('')

foreach ($e in @(@('E1', 'E1_varredura.txt'), @('E2', 'E2_registros.txt'))) {
    $ls = foreach ($b in 4, 8, 16, 32, 64) {
        [pscustomobject]@{ Chave = $b; R = Invoke-Exp "$($e[0])_bloco$b" (Join-Path $exp $e[1]) 256 $b 1 wb }
    }
    Add-Tabela "$($e[0]) - $($e[1]) (256 B, 1 via, WB)" 'Bloco (B)' $ls -Lidos
}

$cfgVias = @(
    @('E3', 'E3_dois_vetores.txt', $exp),
    @('E3mod', 'E3_dois_vetores_modificado.txt', $exp),
    @('E4', 'E1_varredura.txt', $exp),
    @('E5_80', 'E5_laco_80.txt', $exp),
    @('E5_60', 'E5_laco_60.txt', $exp)
)
foreach ($e in $cfgVias) {
    $ls = foreach ($v in 1, 2, 4, 16) {
        [pscustomobject]@{ Chave = $v; R = Invoke-Exp "$($e[0])_vias$v" (Join-Path $e[2] $e[1]) 256 16 $v wb }
    }
    Add-Tabela "$($e[0]) - $($e[1]) (256 B, bloco 16 B, WB)" 'Vias' $ls
}

foreach ($t in 'E6_acumulador', 'E6_copia', 'E6_esparsa') {
    $ls = foreach ($p in 'wt', 'wb') {
        [pscustomobject]@{ Chave = $p.ToUpper(); R = Invoke-Exp "${t}_$p" (Join-Path $exp "$t.txt") 256 16 2 $p }
    }
    Add-Tabela "$t (256 B, bloco 16 B, 2 vias)" 'Politica' $ls -Escrita
}

$ls = foreach ($v in 1, 2, 4) {
    [pscustomobject]@{ Chave = $v; R = Invoke-Exp "Desafio_vias$v" (Join-Path $des 'desafio.txt') 256 16 $v wb }
}
Add-Tabela 'Desafio - desafio.txt (256 B, bloco 16 B, WB)' 'Vias' $ls

$md | Out-File (Join-Path $out 'tabelas.md') -Encoding utf8
Write-Host "Saidas gravadas em $out"

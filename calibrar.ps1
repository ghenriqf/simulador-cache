# Roda os quatro traços de calibração e compara com os arquivos *_esperado.txt.
# Uso: powershell -ExecutionPolicy Bypass -File calibrar.ps1
$ErrorActionPreference = 'Stop'
$raiz = $PSScriptRoot
$sim  = Join-Path $raiz 'target\release\simcache.exe'
$cal  = Join-Path $raiz 'tracos\calibracao'

if (-not (Test-Path $sim)) {
    Push-Location $raiz
    cargo build --release --quiet
    Pop-Location
}

function Invoke-Sim([string]$traco, [string[]]$opts) {
    (& $sim (Join-Path $cal $traco) @opts) -join "`n"
}
function Format-Norm([string]$s) {
    ((($s -replace "`r", '') -split "`n" | ForEach-Object { $_.TrimEnd() }) -join "`n").Trim()
}

$sep = "`n`n" + ('=' * 70) + "`n`n"
$casos = @(
    @{ Nome = 'C1_mapeamento_direto'; Saida = { Invoke-Sim 'C1_mapeamento_direto.txt' @('-c','128','-b','16','-v','1','-p','wb') } },
    @{ Nome = 'C2_associativa_LRU';   Saida = { Invoke-Sim 'C2_associativa_LRU.txt'   @('-c','64','-b','16','-v','2','-p','wb') } },
    @{ Nome = 'C3_escrita';           Saida = { (Invoke-Sim 'C3_escrita.txt' @('-c','64','-b','16','-v','1','-p','wb')) + $sep +
                                                (Invoke-Sim 'C3_escrita.txt' @('-c','64','-b','16','-v','1','-p','wt')) } },
    @{ Nome = 'C4_classificacao';     Saida = { Invoke-Sim 'C4_classificacao.txt'     @('-c','64','-b','16','-v','1','-p','wb') } }
)

$falhas = 0
foreach ($c in $casos) {
    $obtido   = Format-Norm (& $c.Saida)
    $esperado = Format-Norm (Get-Content -Raw -Encoding UTF8 (Join-Path $cal "$($c.Nome)_esperado.txt"))
    if ($obtido -ceq $esperado) {
        Write-Host ("OK     {0}" -f $c.Nome) -ForegroundColor Green
    } else {
        $falhas++
        Write-Host ("FALHOU {0}" -f $c.Nome) -ForegroundColor Red
        Compare-Object ($esperado -split "`n") ($obtido -split "`n") -SyncWindow 0 |
            ForEach-Object { '   {0} {1}' -f ($(if ($_.SideIndicator -eq '<=') { 'esperado:' } else { 'obtido:  ' })), $_.InputObject }
    }
}
if ($falhas -eq 0) { Write-Host "`nCalibracao completa: 4/4 identicos." } else { exit 1 }

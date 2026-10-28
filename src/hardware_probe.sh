# Read-only, dependency-optional probe, run inside the existing 30s deadline.
# Keep the legacy summary fields for older views and observation consumers.
PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/local/cuda/bin:/usr/lib/wsl/lib:$PATH"
export PATH
LC_ALL=C
export LC_ALL
printf 'source_time=%s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ' 2>/dev/null || echo unknown)"
printf 'hostname=%s\n' "$(hostname 2>/dev/null || echo unknown)"
printf 'system=%s\n' "$(uname -srm 2>/dev/null || uname -a 2>/dev/null || echo unknown)"
printf 'architecture=%s\n' "$(uname -m 2>/dev/null)"
printf 'uptime=%s\n' "$(uptime 2>/dev/null | sed 's/^ *//')"
if [ -r /proc/loadavg ]; then
  awk '{printf "load=%s %s %s\n", $1, $2, $3}' /proc/loadavg
else
  printf 'load=%s\n' "$(sysctl -n vm.loadavg 2>/dev/null | tr -d '{}')"
fi
if command -v free >/dev/null 2>&1; then
  free -m | awk '/^Mem:/ && $2 > 0 {printf "memory=%s/%s MB (%d%%)\n", $3, $2, ($3 * 100) / $2}'
elif command -v memory_pressure >/dev/null 2>&1; then
  memory_pressure 2>/dev/null | awk '/System-wide memory free percentage/ {printf "memory=%s free\n", $5}'
else
  printf 'memory=unknown\n'
fi
df -hP / 2>/dev/null | awk 'NR==2 {printf "storage=%s/%s %s used\n", $3, $2, $5}'
if command -v ps >/dev/null 2>&1; then
  printf 'tasks=%s processes\n' "$(ps -e 2>/dev/null | wc -l | tr -d ' ')"
else
  printf 'tasks=unknown\n'
fi

tersh_os=$(uname -s 2>/dev/null)
case "$tersh_os" in
  Linux)
    printf 'hardware_model=%s\n' "$(cat /sys/class/dmi/id/sys_vendor /sys/class/dmi/id/product_name 2>/dev/null | tr '\n' ' ')"
    awk -F: '/^(model name|Hardware|Processor)[[:space:]]*:/ {sub(/^[[:space:]]+/, "", $2); print "cpu_model=" $2; exit}' /proc/cpuinfo 2>/dev/null
    # Count physical (socket, core) pairs; ARM/VMs without IDs remain unknown.
    awk -F: '
      /^physical id[[:space:]]*:/ {socket=$2}
      /^core id[[:space:]]*:/ {core=$2}
      /^$/ {if (socket != "" && core != "") seen[socket ":" core]=1; socket=""; core=""}
      END {if (socket != "" && core != "") seen[socket ":" core]=1; for (pair in seen) n++; if (n) print "cpu_cores=" n}
    ' /proc/cpuinfo 2>/dev/null
    printf 'cpu_threads=%s\n' "$(getconf _NPROCESSORS_ONLN 2>/dev/null)"
    awk '
      /^MemTotal:/ {total=$2}
      /^MemAvailable:/ {available=$2; have_available=1}
      END {
        if (total > 0) printf "memory_total_bytes=%.0f\n", total * 1024;
        if (total > 0 && have_available && available <= total) printf "memory_used_bytes=%.0f\n", (total-available) * 1024;
        print "memory_basis=Linux MemTotal - MemAvailable (reclaimable cache excluded)"
      }
    ' /proc/meminfo 2>/dev/null
    awk '/^cpu / {printf "cpu_ticks_start="; for(i=2;i<=9;i++) printf "%s%s", $i, (i==9 ? "\n" : " ")}' /proc/stat 2>/dev/null
    sleep 1
    awk '/^cpu / {printf "cpu_ticks_end="; for(i=2;i<=9;i++) printf "%s%s", $i, (i==9 ? "\n" : " ")}' /proc/stat 2>/dev/null
    ;;
  Darwin)
    printf 'hardware_model=%s\n' "$(sysctl -n hw.model 2>/dev/null)"
    printf 'cpu_model=%s\n' "$(sysctl -n machdep.cpu.brand_string 2>/dev/null)"
    printf 'cpu_cores=%s\n' "$(sysctl -n hw.physicalcpu 2>/dev/null)"
    printf 'cpu_threads=%s\n' "$(sysctl -n hw.logicalcpu 2>/dev/null)"
    tersh_ram=$(sysctl -n hw.memsize 2>/dev/null)
    printf 'memory_total_bytes=%s\n' "$tersh_ram"
    vm_stat 2>/dev/null | awk -v total="$tersh_ram" '
      /page size of/ {for(i=1;i<=NF;i++) if($i=="of") page=$(i+1)}
      /^Pages free:/ {free=$3+0; have_free=1}
      /^Pages speculative:/ {speculative=$3+0}
      END {if (total > 0 && page > 0 && have_free && (free+speculative)*page <= total)
        printf "memory_used_bytes=%.0f\n", total-(free+speculative)*page}
    '
    printf 'memory_basis=macOS physical occupied; includes caches, not memory pressure\n'
    # top sample 1 is since boot; sample 2 is the requested one-second interval.
    top -l 2 -s 1 -n 0 2>/dev/null | awk '
      /^CPU usage:/ {samples++; for(i=1;i<=NF;i++) if($i=="idle") {idle=$(i-1); sub(/%$/, "", idle)}}
      END {if(samples >= 2 && idle ~ /^[0-9.]+$/ && (idle+0) >= 0 && (idle+0) <= 100) printf "cpu_usage=%.2f\n", 100-idle}
    '
    ;;
esac

# One NVIDIA query serves both the legacy summary and each individual GPU.
tersh_nvidia=
if command -v nvidia-smi >/dev/null 2>&1; then
  tersh_nvidia=$(nvidia-smi --query-gpu=index,name,utilization.gpu,memory.used,memory.total,temperature.gpu --format=csv,noheader,nounits 2>/dev/null | awk -F, 'NF==6 && $1 ~ /^[[:space:]]*[0-9]+[[:space:]]*$/')
fi
if [ -n "$tersh_nvidia" ]; then
  printf '%s\n' "$tersh_nvidia" | awk -F, '
    function bytes(v) {return v ~ /^[0-9]+([.][0-9]+)?$/ ? sprintf("%.0f", v*1048576) : ""}
    NF==6 {
      for(i=1;i<=NF;i++) {gsub(/^[[:space:]]+|[[:space:]]+$/, "", $i); gsub(/\|/, "/", $i)}
      printf "hardware_gpu=%s|%s|%s|%s|%s|%s|dedicated\n", $1,$2,$3,bytes($4),bytes($5),$6;
      legacy=legacy sep $2 ", " $3 ", " $4 ", " $5; sep=";"
    }
    END {if (legacy != "") print "gpu=" legacy; else print "gpu=unknown:driver_query_failed"}
  '
elif [ "$tersh_os" = Darwin ]; then
  # IOAccelerator exposes Apple GPU counters without root. Other driver names
  # may still provide inventory with unknown utilization/VRAM.
  tersh_apple=$(ioreg -r -c IOAccelerator -d 1 2>/dev/null | awk '
    function emit() {
      if (model != "") {
        gsub(/\|/, "/", model);
        shared=(model ~ /^Apple /);
        printf "hardware_gpu=%d|%s|%s|%s|%s||%s\n", gpu_index++, model, use, (shared ? shared_used : vram_used), (shared ? "" : vram_total), (shared ? "shared" : "dedicated")
      }
      model=""; use=""; shared_used=""; vram_used=""; vram_total=""
    }
    /^\+-o / {emit()}
    /"model" = "/ {model=$0; sub(/^.*"model" = "/, "", model); sub(/".*$/, "", model)}
    /"PerformanceStatistics"/ {
      n=split($0, fields, ",");
      for(i=1;i<=n;i++) {
        field=fields[i]; value=field; sub(/^.*=/, "", value); sub(/[^0-9].*$/, "", value);
        if(field ~ /"Device Utilization %"=/) use=value;
        if(field ~ /"In use system memory"=/) shared_used=value;
        if(field ~ /"vramUsedBytes"=/) vram_used=value;
        if(field ~ /"vramTotalBytes"=/) vram_total=value;
      }
    }
    END {emit()}
  ')
  if [ -n "$tersh_apple" ]; then
    printf '%s\n' "$tersh_apple"
  else
    # Inventory fallback for drivers that expose no accelerator model.
    system_profiler SPDisplaysDataType -detailLevel mini 2>/dev/null | awk '
      /Chipset Model:/ {sub(/^.*Chipset Model: */, ""); gsub(/\|/, "/"); printf "hardware_gpu=%d|%s|||||%s\n", gpu_index++, $0, ($0 ~ /^Apple / ? "shared" : "dedicated")}
    '
  fi
  printf 'gpu=unknown:see_hardware_driver_counters\n'
else
  printf 'gpu=unknown:command_unavailable\n'
fi
if [ "$tersh_os" = Linux ]; then
  # DRM inventory works without lspci; AMD drivers may expose utilization/VRAM.
  for tersh_card in /sys/class/drm/card[0-9]*; do
    tersh_id=${tersh_card##*/}
    case "${tersh_id#card}" in ''|*[!0-9]*) continue ;; esac
    [ -d "$tersh_card/device" ] || continue
    tersh_vendor=$(cat "$tersh_card/device/vendor" 2>/dev/null)
    tersh_device_id=$(cat "$tersh_card/device/device" 2>/dev/null)
    # WSL can expose a faux/vgem display node without a physical GPU identity.
    # Leave unidentified adapters unknown instead of counting an extra "GPU".
    [ -n "$tersh_vendor$tersh_device_id" ] || continue
    # NVIDIA was already enumerated above; retain Intel/AMD on mixed hosts.
    if [ -n "$tersh_nvidia" ] && [ "$tersh_vendor" = 0x10de ]; then continue; fi
    case "$tersh_vendor" in
      0x10de) tersh_vendor=NVIDIA ;;
      0x1002) tersh_vendor=AMD ;;
      0x8086) tersh_vendor=Intel ;;
      '') tersh_vendor=GPU ;;
    esac
    tersh_name="$tersh_vendor $tersh_device_id"
    if command -v lspci >/dev/null 2>&1; then
      tersh_pci=$(readlink -f "$tersh_card/device" 2>/dev/null)
      tersh_pci=${tersh_pci##*/}
      if [ -n "$tersh_pci" ]; then
        tersh_pci_name=$(lspci -s "$tersh_pci" 2>/dev/null | sed -n '1{s/^[^ ]* //; s/|/\//g; p;}')
        [ -z "$tersh_pci_name" ] || tersh_name=$tersh_pci_name
      fi
    fi
    printf 'hardware_gpu=%s|%s|%s|%s|%s||dedicated\n' "$tersh_id" "$tersh_name" \
      "$(cat "$tersh_card/device/gpu_busy_percent" 2>/dev/null)" \
      "$(cat "$tersh_card/device/mem_info_vram_used" 2>/dev/null)" \
      "$(cat "$tersh_card/device/mem_info_vram_total" 2>/dev/null)"
  done
fi
printf 'gpu_status=Missing driver counters are unknown, not proof of absent or idle hardware\n'
exit 0

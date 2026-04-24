#!/bin/bash
#
# bendy-web-sential 自动驱动脚本 v3.1
# 功能：通过文件状态机驱动 Claude Code 自动开发
# 机制：写指令到文件 → Claude Code 读指令执行 → 完成标记 → 脚本驱动下一阶段
#

set -e

PROJECT_ROOT="/myproject/rust/bendy-web-sential"
PLAN_FILE="$PROJECT_ROOT/plan.md"

# 状态文件
STATUS_FILE="$PROJECT_ROOT/.dev-status"
INSTRUCTION_FILE="$PROJECT_ROOT/.dev-instruction"
LOG_FILE="$PROJECT_ROOT/logs/auto-dev.log"

# 阶段定义
PHASES=("Phase 1" "Phase 2" "Phase 3" "Phase 4" "Phase 5")

declare -A PHASE_NAMES=(
    ["Phase 1"]="核心基础构建"
    ["Phase 2"]="流量控制模块"
    ["Phase 3"]="管理后台与 UI"
    ["Phase 4"]="安全与生产加固"
    ["Phase 5"]="自动化与灾备"
)

declare -A PHASE_VERSIONS=(
    ["Phase 1"]="0.2.0"
    ["Phase 2"]="0.3.0"
    ["Phase 3"]="0.4.0"
    ["Phase 4"]="0.5.0"
    ["Phase 5"]="0.6.0"
)

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# ============ 日志 ============
log() {
    local level=$1
    local message=$2
    mkdir -p "$(dirname "$LOG_FILE")"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [$level] $message" >> "$LOG_FILE"

    case $level in
        "INFO")  echo -e "${BLUE}[INFO]${NC} $message" ;;
        "OK")    echo -e "${GREEN}[OK]${NC} $message" ;;
        "WARN")  echo -e "${YELLOW}[WARN]${NC} $message" ;;
        "ERROR") echo -e "${RED}[ERROR]${NC} $message" ;;
    esac
}

# ============ 状态管理 ============
get_status() {
    grep "^${1}=" "$STATUS_FILE" 2>/dev/null | cut -d'=' -f2- || echo ""
}

set_status() {
    local key=$1
    local value=$2
    if grep -q "^${key}=" "$STATUS_FILE" 2>/dev/null; then
        sed -i "s|^${key}=.*|${key}=${value}|" "$STATUS_FILE"
    else
        echo "${key}=${value}" >> "$STATUS_FILE"
    fi
}

# ============ 阶段检查 ============
is_phase_done() {
    local num=$1
    grep -q "Phase $num:.*✅ 已完成" "$PLAN_FILE" 2>/dev/null
}

get_next_phase_num() {
    for i in {1..5}; do
        if ! is_phase_done $i; then
            echo "$i"
            return
        fi
    done
    echo "0"  # 0 表示全部完成
}

# ============ 指令生成 ============
generate_instruction() {
    local phase_num=$1
    local phase="${PHASES[$((phase_num - 1))]}"
    local name="${PHASE_NAMES[$phase]}"
    local version="${PHASE_VERSIONS[$phase]}"

    cat > "$INSTRUCTION_FILE" << EOF
================================================================================
                    bendy-web-sential 自动开发指令
================================================================================

时间: $(date '+%Y-%m-%d %H:%M:%S')
状态: 待执行
阶段: $phase
版本: v$version
名称: $name

================================================================================
                              任务列表
================================================================================

请执行以下步骤:

1. 创建分支
   git checkout dev
   git checkout -b feat/phase${phase_num}-$(echo $name | tr ' ' '-' | tr '[:upper:]' '[:lower:]')

2. 按 plan.md 中 $phase 的任务列表开发

3. 完成开发后执行:
   ./scripts/auto-dev.sh --mark-done $phase_num

================================================================================
                              注意事项
================================================================================

- 每次 commit 使用 Conventional Commits 规范
- 完成开发后确保: cargo build --release 通过
- 更新 plan.md 标记阶段完成
- 合并到 dev 后打 tag: v$version
- 运行 ./scripts/auto-dev.sh --mark-done $phase_num

================================================================================
EOF

    log "INFO" "指令已生成: $INSTRUCTION_FILE"
    cat "$INSTRUCTION_FILE"
}

# ============ 阶段完成 ============
mark_phase_done() {
    local phase_num=$1
    local version="${PHASE_VERSIONS[Phase $phase_num]}"

    log "INFO" "标记 Phase $phase_num 为完成..."

    # 更新 plan.md
    case $phase_num in
        1)
            sed -i 's/Phase 1: 核心基础构建 (v0.2.x) — 开发中/Phase 1: 核心基础构建 (v0.2.0) — ✅ 已完成/' "$PLAN_FILE"
            ;;
        2)
            sed -i 's/Phase 2: 流量控制模块 (v0.3.x)/Phase 2: 流量控制模块 (v0.3.0) — ✅ 已完成/' "$PLAN_FILE"
            ;;
        3)
            sed -i 's/Phase 3: 管理后台与 UI (v0.4.x)/Phase 3: 管理后台与 UI (v0.4.0) — ✅ 已完成/' "$PLAN_FILE"
            ;;
        4)
            sed -i 's/Phase 4: 安全与生产加固 (v0.5.x)/Phase 4: 安全与生产加固 (v0.5.0) — ✅ 已完成/' "$PLAN_FILE"
            ;;
        5)
            sed -i 's/Phase 5: 自动化与灾备 (v0.6.x)/Phase 5: 自动化与灾备 (v0.6.0) — ✅ 已完成/' "$PLAN_FILE"
            ;;
    esac

    # 更新状态
    set_status "last_completed" "$(date +%Y-%m-%dT%H:%M:%S)"
    set_status "phase_${phase_num}_done" "true"

    # 提交
    cd "$PROJECT_ROOT"
    git add plan.md 2>/dev/null || true
    git commit -m "chore(release): v$version Phase $phase_num completed" 2>/dev/null || true

    log "OK" "Phase $phase_num 完成标记已提交"

    # 驱动下一阶段
    local next=$(get_next_phase_num)
    if [ "$next" != "0" ]; then
        echo ""
        log "INFO" "驱动下一阶段..."
        sleep 2
        generate_instruction "$next"
    else
        echo ""
        log "OK" "所有阶段已完成！"
        set_status "all_complete" "true"
        cat > "$INSTRUCTION_FILE" << EOF
================================================================================
                    bendy-web-sential 开发完成
================================================================================

时间: $(date '+%Y-%m-%d %H:%M:%S')
状态: ✅ 全部完成

所有 5 个阶段已全部完成！

最终版本:
- Phase 1: v0.2.0 ✅
- Phase 2: v0.3.0 ✅
- Phase 3: v0.4.0 ✅
- Phase 4: v0.5.0 ✅
- Phase 5: v0.6.0 ✅

请通知项目负责人进行最终验收。
EOF
    fi
}

# ============ 显示 ============
show_status() {
    echo ""
    echo "=============================================="
    echo "   bendy-web-sential 自动驱动脚本 v3.1"
    echo "=============================================="
    echo ""

    echo "阶段进度:"

    for i in {1..5}; do
        local phase="${PHASES[$((i - 1))]}"
        local name="${PHASE_NAMES[$phase]}"
        local version="${PHASE_VERSIONS[$phase]}"

        if is_phase_done $i; then
            echo -e "  ${GREEN}✓${NC} Phase $i v$version - $name (已完成)"
        elif [ $i -eq $(get_next_phase_num) ]; then
            echo -e "  ${YELLOW}→${NC} Phase $i v$version - $name (待开发)"
        else
            echo -e "  ${RED}○${NC} Phase $i v$version - $name (待开发)"
        fi
    done

    echo ""
    echo "当前状态:"

    local last=$(get_status "last_completed")
    local all_done=$(get_status "all_complete")

    if [ "$all_done" == "true" ]; then
        echo -e "  ${GREEN}所有阶段已完成${NC}"
    elif [ -n "$last" ]; then
        echo -e "  最后完成: $last"
    fi

    local next=$(get_next_phase_num)
    if [ "$next" != "0" ]; then
        echo ""
        echo "下一步: 在 Claude Code 中输入以下指令启动开发:"
        echo -e "  ${CYAN}开始开发 Phase $next${NC}"
    fi

    echo ""
}

show_instruction() {
    if [ -f "$INSTRUCTION_FILE" ]; then
        echo ""
        echo "当前指令文件内容:"
        echo "=============================================="
        cat "$INSTRUCTION_FILE"
        echo "=============================================="
    fi
}

# ============ 命令 ============
cmd_next() {
    local next=$(get_next_phase_num)

    if [ "$next" == "0" ]; then
        echo -e "${GREEN}所有阶段已完成！${NC}"
        return
    fi

    generate_instruction "$next"
}

cmd_mark_done() {
    local phase_num=$1

    if [ -z "$phase_num" ]; then
        echo "请指定阶段编号: --mark-done <1-5>"
        echo "例如: ./auto-dev.sh --mark-done 1"
        exit 1
    fi

    if [ $phase_num -lt 1 ] || [ $phase_num -gt 5 ]; then
        echo "阶段编号必须在 1-5 之间"
        exit 1
    fi

    mark_phase_done "$phase_num"
}

cmd_reset() {
    echo -e "${YELLOW}警告: 将重置所有状态${NC}"
    read -p "确认? (y/n): " confirm

    if [ "$confirm" == "y" ]; then
        > "$STATUS_FILE"
        > "$INSTRUCTION_FILE"
        log "OK" "状态已重置"
    fi
}

cmd_help() {
    cat << EOF
bendy-web-sential 自动驱动脚本 v3.1

用法: $0 <命令>

命令:
    status           显示当前进度（默认）
    next             生成下一阶段指令
    --mark-done <n>  标记阶段 n 完成，驱动下一阶段
    reset            重置所有状态
    help             显示帮助

使用流程:
    1. 运行 ./scripts/auto-dev.sh next 生成指令
    2. 在 Claude Code 中执行指令
    3. 完成开发后运行 ./scripts/auto-dev.sh --mark-done <阶段号>
    4. 脚本自动生成下一阶段指令
    5. 重复直到所有阶段完成

文件说明:
    $STATUS_FILE      状态文件
    $INSTRUCTION_FILE 当前指令文件
    $LOG_FILE         日志文件

EOF
}

# ============ 主程序 ============
main() {
    mkdir -p "$(dirname "$LOG_FILE")" 2>/dev/null || true
    touch "$STATUS_FILE" 2>/dev/null || true

    case "${1:-}" in
        status)
            show_status
            ;;
        next)
            cmd_next
            ;;
        --mark-done|-m)
            cmd_mark_done "${2:-}"
            ;;
        reset)
            cmd_reset
            ;;
        help|--help|-h)
            cmd_help
            ;;
        *)
            show_status
            show_instruction
            ;;
    esac
}

main "$@"
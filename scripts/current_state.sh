#!/bin/bash

run_script_multiple_times() {
    local script_path="$1"
    local count="$2"
    count=$(printf "%d" "$count" 2>/dev/null)

    # Validate inputs
    if [ ! -f "$script_path" ]; then
        echo "Error: Script '$script_path' not found" >&2
        return 1
    fi
    
    if [ ! -x "$script_path" ]; then
        echo "Error: Script '$script_path' is not executable" >&2
        return 1
    fi
    
    if ! [[ "$count" =~ ^[0-9]+$ ]]; then
        echo "Error: Count must be a positive integer" >&2
        return 1
    fi
    
    if [ "$count" -le 0 ]; then
        echo "Error: Count must be greater than 0" >&2
        return 1
    fi

    # Run the script the specified number of times
    for i in $(seq $count); do
        "./$script_path"
        printf "\n"
    done
}

run_script_multiple_times "./inserts/plant/create_plant_hab.sh" 45
run_script_multiple_times "./inserts/plant/create_plant_fat.sh" 8
run_script_multiple_times "./inserts/plant/create_plant_cay.sh" 8
run_script_multiple_times "./inserts/plant/create_plant_reap.sh" 14


# for i in {1..45}; do ; done
# for i in {1..8}; do ./inserts/plant/create_plant_cay.sh; done
# for i in {1..8}; do ./inserts/plant/create_plant_fat.sh; done
# for i in {1..14}; do ./inserts/plant/create_plant_reap.sh; done

#!/bin/bash

icon="⭕"

read -a state <<< "$(pomoctl state)"
case "${state[0]}" in
    stopped)
        state_color="#FFF"
        ;;
    working)
        state_color="#F1C232"
        ;;
    short-break)
        state_color="#32F29F"
        ;;
    long-break)
        state_color="#2AEA2A"
        ;;
esac
echo "%{T4}%{F$state_color}$icon%{F-}%{T-} ${state[1]}"

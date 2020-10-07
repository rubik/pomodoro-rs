#!/bin/bash

icon="⭕"

read -a state <<< "$(pomoctl state)"
case "${state[0]}" in
    connection-error)
        state_color="#FFF"
        ;;
    stopped)
        state_color="#FFF"
        ;;
    working)
        state_color="#F1C232"
        ;;
    short-break)
        state_color="#2AEA2A"
        ;;
    long-break)
        state_color="#2AEA2A"
        icon="⬤ "
        ;;
esac
echo "%{T4}%{F$state_color}$icon%{F-}%{T-} ${state[1]}"

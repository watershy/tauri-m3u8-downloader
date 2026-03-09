// src/composables/useThroughputGraph.ts
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { globalTotalSpeedMb } from './useJobs';

export function useThroughputGraph(maxDataPoints: number = 60) {
    const speedHistory = ref<number[]>(new Array(maxDataPoints).fill(0));
    let tickInterval: ReturnType<typeof setInterval> | null = null;

    onMounted(() => {
        // Sample the global speed every 1 second, even if it sits at 0
        tickInterval = setInterval(() => {
            speedHistory.value.shift();
            speedHistory.value.push(globalTotalSpeedMb.value);
        }, 1000);
    });

    onUnmounted(() => {
        if (tickInterval) clearInterval(tickInterval);
    });

    const smartMetrics = computed(() => {
        const currentMb = globalTotalSpeedMb.value;
        const rawMaxMb = Math.max(0, ...speedHistory.value, currentMb);

        let unit = 'MB/s';
        let multiplier = 1;
        let maxLabelVal = 10; 

        if (rawMaxMb > 0 && rawMaxMb < 1.0) {
            unit = 'KB/s';
            multiplier = 1024;
            const rawMaxKb = rawMaxMb * 1024;

            if (rawMaxKb <= 10) maxLabelVal = 10;
            else if (rawMaxKb <= 50) maxLabelVal = Math.ceil(rawMaxKb / 10) * 10;
            else if (rawMaxKb <= 100) maxLabelVal = Math.ceil(rawMaxKb / 20) * 20;
            else if (rawMaxKb <= 500) maxLabelVal = Math.ceil(rawMaxKb / 50) * 50;
            else maxLabelVal = Math.ceil(rawMaxKb / 100) * 100;
        } 
        else {
            unit = 'MB/s';
            multiplier = 1;

            if (rawMaxMb <= 10) maxLabelVal = 10;
            else if (rawMaxMb <= 50) maxLabelVal = Math.ceil(rawMaxMb / 10) * 10;
            else if (rawMaxMb <= 100) maxLabelVal = Math.ceil(rawMaxMb / 20) * 20;
            else maxLabelVal = Math.ceil(rawMaxMb / 50) * 50;
        }

        return {
            maxLabel: `${maxLabelVal} ${unit}`,
            instantLabel: `${(currentMb * multiplier).toFixed(1)} ${unit}`,
            totalSpeedLabel: `${(currentMb * multiplier).toFixed(2)} ${unit}`,
            ceilingMb: maxLabelVal / multiplier 
        };
    });

    const graphPath = computed(() => {
        const maxVal = smartMetrics.value.ceilingMb; 
        
        let path = `M 0 40 `; 
        speedHistory.value.forEach((val, index) => {
            const x = (index / (maxDataPoints - 1)) * 100;
            const y = 40 - ((val / maxVal) * 40);
            path += `L ${x} ${y} `;
        });

        path += `L 100 40 Z`; 
        return path;
    });

    return {
        smartMetrics,
        graphPath
    };
}
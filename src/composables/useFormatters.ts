export function useFormatters() {
    const formatSize = (bytes: number): string => {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    };

    const formatTime = (seconds: number | null): string => {
        if (!seconds || seconds === Infinity) return '--:--';
        const m = Math.floor(seconds / 60);
        const s = Math.floor(seconds % 60);
        return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    };

    const formatEtaText = (seconds: number | null): string => {
        // Handle the initial state before we have enough data to calculate speed
        if (seconds === null || seconds === undefined || seconds === Infinity) {
            return '--';
        }

        if (seconds <= 0) return '--';

        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = Math.floor(seconds % 60);
        const parts = [];
        if (h > 0) {
            parts.push(`${h} hr${h > 1 ? 's' : ''}`);
        }
        if (m > 0) {
            parts.push(`${m} min${m > 1 ? 's' : ''}`);
        }

        if (s > 0 && h === 0) {
            parts.push(`${s} sec${s > 1 ? 's' : ''}`);
        }

        return parts.join(' ');
    };

    const quantizeSeconds = (rawSeconds: number | null): number | null => {
        if (!rawSeconds || rawSeconds === Infinity) return rawSeconds;
        if (rawSeconds > 900) return Math.round(rawSeconds / 300) * 300; // > 15m: 5m buckets
        if (rawSeconds > 300) return Math.round(rawSeconds / 60) * 60;   // > 5m: 1m buckets
        if (rawSeconds > 60) return Math.round(rawSeconds / 15) * 15;    // > 1m: 15s buckets
        if (rawSeconds > 15) return Math.round(rawSeconds / 5) * 5;      // > 15s: 5s buckets
        return Math.round(rawSeconds);                                   // Exact seconds
    };

    const formatStatus = (status: any): string => {
        if (typeof status === 'string') {
            if (status === 'CompletedSuccess') return 'Completed';
            return status;
        }
        if (status && status.CompletedError) {
            return 'Error';
        }
        return 'Unknown';
    };

    const getStatusSeverity = (status: any): string => {
        const s = typeof status === 'string' ? status : (status?.CompletedError ? 'Error' : 'Unknown');

        switch (s) {
            case 'Downloading': return 'info';
            case 'Paused': return 'warning';
            case 'Merging': return 'warning';
            case 'CompletedSuccess': return 'success';
            case 'Error': return 'danger';
            default: return 'secondary';
        }
    };

    return {
        formatSize,
        formatTime,
        formatEtaText,
        quantizeSeconds,
        formatStatus,
        getStatusSeverity
    };
}
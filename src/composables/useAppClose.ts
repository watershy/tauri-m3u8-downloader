import { appWindow } from '@tauri-apps/api/window';
import { exit } from '@tauri-apps/api/process';
import { invoke } from '@tauri-apps/api/tauri';
import { useConfirm } from 'primevue/useconfirm';

export function useAppClose() {
    const confirm = useConfirm();

    const initCloseInterceptor = async () => {
        await appWindow.onCloseRequested(async (event) => {
            // 1. Instantly prevent default close so the app stays open while we check
            event.preventDefault(); 

            try {
                // 2. Fetch the absolute latest state from Rust directly
                const jobs: any[] = await invoke('get_jobs');
                const activeJobs = jobs.filter(j => 
                    j.status === 'Downloading' || 
                    j.status === 'Merging' || 
                    j.status === 'Queued'
                );

                // 3. Prompt if there are active jobs
                if (activeJobs.length > 0) {
                    confirm.require({
                        message: 'You have active downloading or merging jobs. If you close the app, they will be paused and merging will be aborted. Are you sure you want to exit?',
                        header: 'Confirm Exit',
                        icon: 'pi pi-exclamation-triangle',
                        acceptClass: 'p-button-danger',
                        acceptLabel: 'Yes, Exit',
                        rejectLabel: 'Cancel',
                        accept: async () => {
                            for (const job of activeJobs) {
                                try {
                                    await invoke('pause_job', { jobId: job.id });
                                } catch (e) {
                                    console.error(`Failed to pause ${job.id}:`, e);
                                }
                            }
                            await new Promise(resolve => setTimeout(resolve, 500));
                            await exit(0);
                        }
                    });
                } else {
                    // No active jobs, safe to close immediately
                    await exit(0);
                }
            } catch (error) {
                console.error("Failed to check jobs on exit:", error);
                await exit(0); // Failsafe: if Rust crashes, just exit
            }
        });
    };

    return { initCloseInterceptor };
}
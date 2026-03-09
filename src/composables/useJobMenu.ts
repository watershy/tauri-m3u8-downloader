import { ref } from 'vue';

// Define the shape of the actions we need
interface MenuActions {
    resume: () => void;
    pause: () => void;
    delete: () => void;
    openFolder: (job: any) => void;
}

export function useJobMenu(selectedJobs: any, actions: MenuActions) {
    const cm = ref(); // Reference to the ContextMenu component
    const menuModel = ref<any[]>([]);

    const onRowContextMenu = (event: any) => {
        const clickedJob = event.data;

        // 1. Smart Selection Logic
        // If we right-click something NOT selected, we select only that one.
        const isAlreadySelected = selectedJobs.value.some((j: any) => j.id === clickedJob.id);
        
        if (!isAlreadySelected) {
            selectedJobs.value = [clickedJob];
        }

        // 2. Calculate Capabilities
        const count = selectedJobs.value.length;
        const hasMultiple = count > 1;

        const anyResumable = selectedJobs.value.some((j: any) => 
            j.status !== 'Downloading' && j.status !== 'Merging' && j.status !== 'CompletedSuccess'
        );
        const anyPausable = selectedJobs.value.some((j: any) => 
            j.status === 'Downloading' || j.status === 'Queued'
        );

        // 3. Build the Menu
        menuModel.value = [
            {
                label: hasMultiple ? `Resume (${count} items)` : 'Resume',
                icon: 'pi pi-play',
                disabled: !anyResumable,
                command: () => actions.resume()
            },
            {
                label: hasMultiple ? `Pause (${count} items)` : 'Pause',
                icon: 'pi pi-pause',
                disabled: !anyPausable,
                command: () => actions.pause()
            },
            {
                label: 'Open Folder',
                icon: 'pi pi-folder-open',
                disabled: hasMultiple, // Opening 10 folders at once is annoying
                command: () => actions.openFolder(clickedJob)
            },
            { separator: true },
            {
                label: hasMultiple ? `Delete (${count} items)` : 'Delete',
                icon: 'pi pi-trash',
                class: 'text-red-500',
                command: () => actions.delete()
            }
        ];

        // 4. Show the menu
        // We need the original browser event to position the menu
        if (cm.value) {
            cm.value.show(event.originalEvent);
        }
    };

    return {
        cm,
        menuModel,
        onRowContextMenu
    };
}
<script>
    window.onload = function () {
        let fileList = new Array();
        let totalFiles = 0;

        function formatBytes(bytes, decimals = 2) {
            if (!+bytes) return "0 Bytes";

            const k = 1024;
            const dm = decimals < 0 ? 0 : decimals;
            const sizes = [
                "Bytes",
                "KiB",
                "MiB",
                "GiB",
                "TiB",
                "PiB",
                "EiB",
                "ZiB",
                "YiB",
            ];

            const i = Math.floor(Math.log(bytes) / Math.log(k));

            return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
        }

        function appendFileList(id, name, blob) {
            totalFiles++;
            fileList.unshift({
                id: id,
                name: name,
            });

            let element = document.createElement("tr");
            element.id = `file-${totalFiles.toString()}`;
            element.innerHTML = `
            <tr>
                <th>${id}</th>
                <th>${name}</th>
                <th>type</th>
                <th>${formatBytes(20010)}</th>
                <th>
                    <a download="download" href="url">
                        <button class="download-btn"></button>
                    </a>
                </th>
            </tr>
            `;

            document
                .getElementById("feed-table")
                .insertBefore(
                    element,
                    document.getElementById("feed-table").firstChild,
                );

            if (fileList.length >= 15) {
                fileList.length = 15;
            }
        }

        for (let i = 0; i < 5; i++) {
            appendFileList(125, "aaa.txt", new Blob());
        }
    };
</script>

<main>
    <h2>Live File Feed</h2>

    <table id="feed-table">
        <thead>
            <tr>
                <th>ID</th>
                <th>Name</th>
                <th>Type</th>
                <th>Size</th>
                <th>Download</th>
            </tr>
        </thead>
        <tbody></tbody>
    </table>
</main>

<style>
    main {
        border: 3px solid mediumorchid;
        border-radius: 10px;
    }
</style>

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

        function fadeRow(element) {
            // var height = 0.1; // initial height
            // element.style.display = "block";
            // var timer = setInterval(function () {
            //     if (height >= 1) {
            //         clearInterval(timer);
            //     }
            //     element.style.opacity = op;
            //     element.style.filter = "alpha(opacity=" + op * 100 + ")";
            //     op += op * 0.1;
            // }, 10);
            setTimeout(function () {
                var op = 1; // initial opacity
                var timer = setInterval(function () {
                    if (op <= 0.1) {
                        clearInterval(timer);
                        element.style.display = "none";
                        element.remove();
                    }
                    element.style.opacity = op;
                    element.style.filter = "alpha(opacity=" + op * 100 + ")";
                    op -= op * 0.1;
                }, 50);
            }, 4000);
        }

        function appendFileList(id, name, blob) {
            totalFiles++;
            fileList.unshift({
                id: id,
                name: name,
            });

            let element = document.createElement("tr");
            element.id = `file-${totalFiles.toString()}`;
            element.onload = fadeRow(element);
            element.innerHTML = `
            <th>${id}</th>
            <th>${name}</th>
            <th>${formatBytes(20010)}</th>
            <th>
                <a download="download" href="url">
                    <button class="download-btn"></button>
                </a>
            </th>
            `;

            let tableBody = document.getElementById("feed-table").lastChild;
            tableBody.insertBefore(element, tableBody.firstChild);

            if (fileList.length >= 15) {
                fileList.length = 15;
            }
        }

        for (let i = 0; i < 5; i++) {
            setTimeout(
                function () {
                    appendFileList(125, "aaa.txt", new Blob());
                },
                2000 * i,
                i,
            );
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

    @keyframes file-fade-row {
        0% {
            opacity: 100%;
            height: 100%;
        }
        80% {
            opacity: 100%;
            height: 100%;
        }
        100% {
            opacity: 0%;
            height: 0%;
        }
    }
</style>

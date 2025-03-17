<script>
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
        }, 3500);
    }

    window.appendFileList = function appendFileList(id, name, base64) {
        totalFiles++;
        fileList.unshift({
            id: id,
            name: name,
        });

        let element = document.createElement("tr");
        element.id = `file-${totalFiles.toString()}`;
        element.onload = fadeRow(element);
        element.innerHTML = `
            <td>${id}</td>
            <td>${name}</td>
            <td>${formatBytes(base64.length)}</td>
            <td>
                <a download="download" href="data:text/unknown;base64,${base64}">
                    <button class="download-btn"></button>
                </a>
            </td>
            `;

        let tableBody = document.getElementById("feed-table").lastChild;
        tableBody.insertBefore(element, tableBody.firstChild);

        if (fileList.length >= 15) {
            fileList.length = 15;
        }
    };

    for (let i = 0; i < 1000; i++) {
        setTimeout(
            function () {
                appendFileList(125, "aaa.txt", "aGVsbG8gd29yZA==");
            },
            2000 * i,
            i,
        );
    }
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
    table {
        display: table;
        justify-content: space-evenly;
        margin: auto;
        padding: auto;

        border: 3px solid var(--accent-1);
        border-radius: 10px;
    }
</style>

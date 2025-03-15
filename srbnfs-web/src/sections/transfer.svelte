<script>
</script>

<main>
    <div id="root-ip">
        <h2>Root Server Setup</h2>

        <p>Connection Status: Disconnected</p>

        <input
            type="text"
            name="Root Server IP"
            id="root-ip-input"
            placeholder="XX.XX.XX.XX"
        />
        <input type="button" value="Connect" id="root-ip-submit" onclick={connect}  />
    </div>
    <br />
    <div id="file-upload">
        <h2>Upload File(s)</h2>

        <input type="file" name="Upload File" multiple id="file-upload-input" />
        <br />
        <input type="button" value="Upload" onclick={upload} />
    </div>

    <script>
        function connect() {
            window.rootsrv = new WebSocket("ws://" + document.querySelector("#root-ip-input").value + ":8989");

            window.rootsrv.addEventListener("on", event => {
                console.log("Connected to remote");
            });

            window.rootsrv.onerror = (error) => {
                console.error('WebSocket error:', error);
            };

            window.rootsrv.addEventListener('message', async (event) => {
                let data = await event.data.text();

                console.log('Message from server:', data);

                let packet = JSON.parse(data);

                if (packet["packet_type"] == "RelayFile") {
                    let content = packet["params"]["FileEncoded"];
                    let name = packet["params"]["FileName"];


                    window.appendFileList(0, name, content);
                }
            });




        }

        function _arrayBufferToBase64( buffer ) {
    var binary = '';
    var bytes = new Uint8Array( buffer );
    var len = bytes.byteLength;
    for (var i = 0; i < len; i++) {
        binary += String.fromCharCode( bytes[ i ] );
    }
    return window.btoa( binary );
}


        async function upload() {
            if (window.rootsrv == undefined) {
                alert("Not connected!");
                return;
            }

            let fileIn = document.querySelector("#file-upload-input");

            for (let i = 0; i < fileIn.files.length; i++) {
                let file = fileIn.files[i];

                let buffer = await file.arrayBuffer();
                let content = _arrayBufferToBase64(buffer);

                let packet = JSON.stringify({
                    date_time: Math.floor(Date.now() / 1000),
                    packet_type: "InjectFileIntoRing",
 
                    params: {
                        FileName: file.name,
                        FileEncoded: content
                    }
                }) + "\n";

                const encoder = new TextEncoder();

                window.rootsrv.send(encoder.encode(packet));
            }
        }
</script>
</main>

<style>
</style>

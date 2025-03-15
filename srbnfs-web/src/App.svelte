<script>
  import SecHeader from "./sections/header.svelte";
  import SecTransfer from "./sections/transfer.svelte";
  import SecMap from "./sections/map.svelte";
  import SecFooter from "./sections/footer.svelte";
</script>

<main>
  <SecHeader></SecHeader>
  <br />
  <SecTransfer></SecTransfer>
  <br />
  <SecMap></SecMap>
  <br />
  <SecFooter></SecFooter>
</main>

<style>
  @import url("https://fonts.googleapis.com/css2?family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap");
</style>

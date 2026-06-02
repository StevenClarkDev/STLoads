(function () {
  function setStageScale() {
    var viewportWidth =
      document.documentElement.clientWidth ||
      document.body.clientWidth ||
      window.innerWidth;
    var scale = viewportWidth / 1440;

    document.documentElement.style.setProperty("--stage-scale", scale);
    document.documentElement.style.setProperty(
      "--stage-height",
      4755 * scale + "px"
    );
  }

  setStageScale();
  window.addEventListener("resize", setStageScale);
})();

export function initNavbar() {
  const toggleButton = document.getElementById("navbar-toggle");
  const navLinks = document.getElementById("nav-links");
  const themeToggle = document.getElementById("theme-toggle");

  function setHighlightTheme(theme) {
    const lightLink = document.getElementById("hljs-light");
    const darkLink = document.getElementById("hljs-dark");
    if (!lightLink || !darkLink) return;

    if (theme === "dark") {
      lightLink.disabled = true;
      darkLink.disabled = false;
    } else {
      lightLink.disabled = false;
      darkLink.disabled = true;
    }
  }

  if (toggleButton && navLinks) {
    toggleButton.addEventListener("click", () => {
      const expanded = toggleButton.getAttribute("aria-expanded") === "true";
      navLinks.classList.toggle("active");
      toggleButton.setAttribute("aria-expanded", !expanded);
    });
  }

  if (themeToggle) {
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    const storedTheme = localStorage.getItem("theme");
    const currentTheme = storedTheme || (prefersDark ? "dark" : "light");

    // Terapkan tema awal
    document.documentElement.setAttribute("data-theme", currentTheme);
    setHighlightTheme(currentTheme);

    themeToggle.addEventListener("click", () => {
      const current = document.documentElement.getAttribute("data-theme");
      const next = current === "dark" ? "light" : "dark";
      document.documentElement.setAttribute("data-theme", next);
      localStorage.setItem("theme", next);
      setHighlightTheme(next);
    });
  }
}

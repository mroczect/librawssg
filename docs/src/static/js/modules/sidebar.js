export function initSidebar() {
  const currentPath = window.location.pathname;
  const sidebarLinks = document.querySelectorAll(".sidebar-nav a");
  const sidebar = document.querySelector(".sidebar");
  const overlay = document.getElementById("sidebar-overlay");
  const closeBtn = document.getElementById("sidebar-close");

  sidebarLinks.forEach((link) => {
    const linkPath = link.getAttribute("href");
    if (linkPath && currentPath.endsWith(linkPath)) {
      link.classList.add("active");
      let parent = link.closest("li");
      while (parent) {
        const parentUl = parent.parentElement;
        if (parentUl && parentUl.tagName === "UL") {
          parentUl.style.display = "block";
        }
        parent = parentUl ? parentUl.closest("li") : null;
      }
    }
  });

  const navbarToggle = document.getElementById("navbar-toggle");

  function openSidebar() {
    sidebar.classList.add("open");
    overlay.classList.add("active");
    navbarToggle?.setAttribute("aria-expanded", "true");
  }

  function closeSidebar() {
    sidebar.classList.remove("open");
    overlay.classList.remove("active");
    navbarToggle?.setAttribute("aria-expanded", "false");
  }

  if (navbarToggle && sidebar) {
    navbarToggle.addEventListener("click", () => {
      if (sidebar.classList.contains("open")) {
        closeSidebar();
      } else {
        openSidebar();
      }
    });
  }

  if (closeBtn) closeBtn.addEventListener("click", closeSidebar);
  if (overlay) overlay.addEventListener("click", closeSidebar);

  // Tutup sidebar jika layar di-resize ke desktop
  window.addEventListener("resize", () => {
    if (window.innerWidth > 768 && sidebar.classList.contains("open")) {
      closeSidebar();
    }
  });
}

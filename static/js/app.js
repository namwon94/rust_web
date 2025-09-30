// 팝업 열기
// function showPopup(popupId) {
//     const popup = document.getElementById(popupId);
//     if (popup) {
//         popup.classList.add('active');
//         document.body.style.overflow = 'hidden'; // 배경 스크롤 방지
//     }
// }

function showPopup(url, popupContainerId) {
    fetch(url)
    .then(response => response.text())
    .then(html => {
        const container = document.getElementById(popupContainerId);
        container.innerHTML = html;
        container.classList.add('active');
        document.body.style.overflow = 'hidden';
    });
}

// 팝업 닫기
function closePopup(popupId) {
    const popup = document.getElementById(popupId);
    if (popup) {
        popup.classList.remove('active');
        document.body.style.overflow = ''; // 스크롤 복원
    }
}

// 모든 팝업 닫기
function closeAllPopups() {
    const popups = document.querySelectorAll('.popup-overlay');
    popups.forEach(popup => {
        popup.classList.remove('active');
    });
    document.body.style.overflow = '';
}

// ESC 키로 팝업 닫기
document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape') {
        closeAllPopups();
    }
});

// 배경 클릭으로 팝업 닫기
document.querySelectorAll('.popup-overlay').forEach(overlay => {
    overlay.addEventListener('click', function(e) {
        if (e.target === this) {
            closePopup(this.id);
        }
    });
});
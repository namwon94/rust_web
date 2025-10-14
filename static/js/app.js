function showPopup(url, popupId) {
    fetch(url)
    .then(response => response.text())
    .then(html => {
        const container = document.getElementById(popupId);
        if(!container) {
            console.error("Element with id ${popupId} not found");
        }
        container.innerHTML = html;
        container.style.display = 'block';
        document.body.style.overflow = 'hidden';

        //팝업 내부의 overlay에 active 클래스 추가
        const overlay = container.querySelector('.popup-overlay');
        if(overlay) {
            overlay.classList.add('active');
        }
    })
    .catch(error => {
        console.log("Error loading popup:", error)
    });
}

// 팝업 닫기
function closePopup(popupId) {
    const container = document.getElementById(popupId);
    if (container) {
        container.innerHTML = '';
        container.style.display = 'none';
        document.body.style.overflow = ''; // 스크롤 복원
    }
}

// 모든 팝업 닫기
function closeAllPopups() {
    const container = document.querySelectorAll('.popup-overlay');
    container.forEach(container => {
        container.classList.remove('active');
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

//로그인 시도
function handleLogin(event) {
    event.preventDefault();
    
    const email = document.getElementById('email').value;
    const password = document.getElementById('password').value;
    const remember = document.getElementById('remember').checked;
    
    console.log('로그인 시도:', { email, remember });
    alert('로그인 기능은 백엔드 연결이 필요합니다.');
}
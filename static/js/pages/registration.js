let nicknameChecked = false;

// 별명 중복 확인
async function checkNickname() {
    const nickname = document.getElementById('nickname').value.trim();
    const statusEl = document.getElementById('nicknameStatus');
    
    if (!nickname) {
        statusEl.textContent = '별명을 입력해주세요.';
        statusEl.style.color = 'var(--rust-orange)';
        return;
    }

    try {
        // 실제 API 호출 (예시)
        // const response = await fetch(`/api/check-nickname?nickname=${nickname}`);
        // const data = await response.json();
        
        // 임시 시뮬레이션
        const isAvailable = Math.random() > 0.5;
        
        if (isAvailable) {
            statusEl.textContent = '✓ 사용 가능한 별명입니다.';
            statusEl.style.color = '#059669';
            nicknameChecked = true;
        } else {
            statusEl.textContent = '✗ 이미 사용 중인 별명입니다.';
            statusEl.style.color = 'var(--rust-orange)';
            nicknameChecked = false;
        }
    } catch (error) {
        statusEl.textContent = '확인 중 오류가 발생했습니다.';
        statusEl.style.color = 'var(--rust-orange)';
        nicknameChecked = false;
    }
}

// 비밀번호 보기/숨기기
function togglePasswordVisibility() {
    const passwordInput = document.getElementById('password');
    const toggleIcon = document.getElementById('toggleIcon');
    
    if (passwordInput.type === 'password') {
        passwordInput.type = 'text';
        toggleIcon.textContent = '🙈 숨기기';
    } else {
        passwordInput.type = 'password';
        toggleIcon.textContent = '👁️ 보기';
    }
}

// 페이지 로드 후 실행
document.addEventListener('DOMContentLoaded', function() {
    let nicknameChecked = false;

    // 별명 입력 시 중복확인 초기화
    const nicknameInput = document.getElementById('nickname');
    if (nicknameInput) {
        nicknameInput.addEventListener('input', function() {
            nicknameChecked = false;
            document.getElementById('nicknameStatus').textContent = '';
        });
    }

    // 비밀번호 확인 실시간 검증
    const passwordConfirm = document.getElementById('passwordConfirm');
    if (passwordConfirm) {
        passwordConfirm.addEventListener('input', checkPasswordMatch);
    }
});

// 비밀번호 일치 확인
function checkPasswordMatch() {
    const password = document.getElementById('password').value;
    const passwordConfirm = document.getElementById('passwordConfirm').value;
    const matchStatus = document.getElementById('passwordMatch');
    
    if (passwordConfirm === '') {
        matchStatus.textContent = '';
        return;
    }
    
    if (password === passwordConfirm) {
        matchStatus.textContent = '✅ 비밀번호가 일치합니다';
        matchStatus.style.color = '#4caf50';
    } else {
        matchStatus.textContent = '❌ 비밀번호가 일치하지 않습니다';
        matchStatus.style.color = '#f44336';
    }
}

// 회원가입 처리
async function handleSignup(event) {
    event.preventDefault();
    
    // 별명 중복확인 여부
    /*
    if (!nicknameChecked) {
        alert('별명 중복 확인을 해주세요.');
        return;
    }
    */
    
    // 비밀번호 일치 확인
    const password = document.getElementById('password').value;
    const passwordConfirm = document.getElementById('passwordConfirm').value;
    
    if (password !== passwordConfirm) {
        alert('비밀번호가 일치하지 않습니다.');
        return;
    }
    
    // 비밀번호 유효성 검사
    /*
    const passwordRegex = /^(?=.*[A-Za-z])(?=.*\d)(?=.*[@$!%*#?&])[A-Za-z\d@$!%*#?&]{8,}$/;
    if (!passwordRegex.test(password)) {
        alert('비밀번호는 최소 8자 이상이며, 영문, 숫자, 특수문자를 포함해야 합니다.');
        return;
    }
    */
    
    const formData = {
        email: document.getElementById('email').value,
        name: document.getElementById('name').value,
        nickname: document.getElementById('nickname').value,
        password: password
    };
    
    //console.log('회원가입 데이터:', formData);
    
    try {
        // 실제 API 호출
        const response = await fetch('/api/register', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(formData)
        });
        
        const result = await response.json();
        //console.log('result.success : ,',result.success,' result.message :',result.message);
        if(result.success) {
            alert(result.message);
            window.location.href = '/home';
        }else {
            alert(result.message);
        }
    } catch (error) {
        alert('서버와 연결할 수 없습니다.');
    }
}
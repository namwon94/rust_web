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

// 별명 입력 시 중복확인 상태 리셋
const nicknameInput = document.getElementById('nickname');
if(nicknameInput){
    document.getElementById('nickname').addEventListener('input', function() {
        nicknameChecked = false;
        document.getElementById('nicknameStatus').textContent = '';
    });
}

// 비밀번호 보기/숨기기
function togglePasswordVisibility() {
    const passwordInput = document.getElementById('reg_password');
    const toggleIcon = document.getElementById('toggleIcon');
    
    if (passwordInput.type === 'password') {
        passwordInput.type = 'text';
        toggleIcon.textContent = '🙈 숨기기';
    } else {
        passwordInput.type = 'password';
        toggleIcon.textContent = '👁️ 보기';
    }
}

// 비밀번호 일치 확인
const passwordConfirmInput = document.getElementById('passwordConfirm');
if(passwordConfirmInput){
    document.getElementById('passwordConfirm').addEventListener('input', function() {
        const password = document.getElementById('reg_password').value;
        const passwordConfirm = this.value;
        const matchEl = document.getElementById('passwordMatch');
        
        if (!passwordConfirm) {
            matchEl.textContent = '';
            return;
        }
        
        if (password === passwordConfirm) {
            matchEl.textContent = '✓ 비밀번호가 일치합니다.';
            matchEl.style.color = '#059669';
        } else {
            matchEl.textContent = '✗ 비밀번호가 일치하지 않습니다.';
            matchEl.style.color = 'var(--rust-orange)';
        }
    });
}

// 회원가입 처리
async function handleSignup(event) {
    event.preventDefault();
    
    // 별명 중복확인 여부
    if (!nicknameChecked) {
        alert('별명 중복 확인을 해주세요.');
        return;
    }
    
    // 비밀번호 일치 확인
    const password = document.getElementById('reg_password').value;
    const passwordConfirm = document.getElementById('passwordConfirm').value;
    
    if (password !== passwordConfirm) {
        alert('비밀번호가 일치하지 않습니다.');
        return;
    }
    
    // 비밀번호 유효성 검사
    const passwordRegex = /^(?=.*[A-Za-z])(?=.*\d)(?=.*[@$!%*#?&])[A-Za-z\d@$!%*#?&]{8,}$/;
    if (!passwordRegex.test(password)) {
        alert('비밀번호는 최소 8자 이상이며, 영문, 숫자, 특수문자를 포함해야 합니다.');
        return;
    }
    
    const formData = {
        email: document.getElementById('reg_email').value,
        name: document.getElementById('name').value,
        nickname: document.getElementById('nickname').value,
        password: password
    };
    
    console.log('회원가입 데이터:', formData);
    
    try {
        // 실제 API 호출
        // const response = await fetch('/api/signup', {
        //     method: 'POST',
        //     headers: { 'Content-Type': 'application/json' },
        //     body: JSON.stringify(formData)
        // });
        
        alert('회원가입이 완료되었습니다!');
        window.close();
    } catch (error) {
        alert('회원가입 중 오류가 발생했습니다.');
    }
}
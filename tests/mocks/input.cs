using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Options;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Test.Controllers
{
    [Route("api/[controller]")]
    [ApiController]
    public class TestController : ControllerBase
    {
        private IDbConnectionFactory<testDbContext> _dbFactory;

        public TestController(IDbConnectionFactory<testDbContext> dbFactory)
        {
            _dbFactory = dbFactory;
        }

        [HttpPost()]
        public void AddAdmin(AdminName adminName)
        {
            using (var cx = _dbFactory.CreateContext())
            {
                var enc = new Encryptor();

                var userOid = Guid.NewGuid();
                var encryptedPwd = enc.Encrypt(adminName.Password, userOid.ToString());

                var user = new User()
                {
                    UserOid = userOid,
                    UserName = adminName.UserName,
                    Email = adminName.LoginEmail,
                    Password = encryptedPwd,
                    UserStatusId = 100,
                    StartDate = DateTime.UtcNow,
                };
                cx.Users.Add(user);

                var admin = new Admin()
                {
                    AdminOid = Guid.NewGuid(),
                    UserOid = user.UserOid,
                    StartDate = DateTime.UtcNow,
                };
                cx.Admins.Add(admin);

                cx.SaveChanges();
            }
        }

        [HttpGet("todoUserTasks/{userOid}")]
        public List<UserTaskDetails> getClientTodoTasks(Guid userOid)
        {
            using (var cx = _dbFactory.CreateContext())
            {
                return (from ut in cx.UserTasks
                        join u in cx.Users on ut.UserOid equals u.UserOid
                        join uts in cx.TaskStatuses on ut.TaskStatusId equals uts.TaskStatusId
                        where ut.UserOid == userOid
                        select new UserTaskDetails()
                        {
                            UserOid = u.UserOid,
                            UserTaskOid = ut.UserTaskOid,
                            Name = ut.Name,
                            CompleteDate = ut.CompleteDate.HasValue ? ut.CompleteDate.Value.ToString("yyyy-MM-dd HH:mm:ss") : "",
                            TaskStatus = uts.Name,
                            TaskStatusId = ut.TaskStatusId,
                            StartDate = ut.StartDate,
                            OrderNumber = ut.OrderNumber,
                        }
                    ).OrderBy(t => t.OrderNumber).ToList();
            }
        }

        [HttpGet("userTaskDetails/{userTaskOid}")]
        public UserTaskDetails GetUserTaskDetails(Guid userTaskOid)
        {
            using (var cx = _dbFactory.CreateContext())
            {
                return (from ut in cx.UserTasks
                        join uts in cx.TaskStatuses on ut.TaskStatusId equals uts.TaskStatusId
                        where ut.UserTaskOid == userTaskOid
                        select new UserTaskDetails()
                        {
                            UserOid = ut.UserOid,
                            UserTaskOid = ut.UserTaskOid,
                            Name = ut.Name,
                            CompleteDate = ut.CompleteDate.HasValue ? ut.CompleteDate.Value.ToString("yyyy-MM-dd HH:mm:ss") : "",
                            TaskStatus = uts.Name,
                            TaskStatusId = ut.TaskStatusId,
                            StartDate = ut.StartDate,
                            OrderNumber = ut.OrderNumber,
                        }).SingleOrDefault();
            }
        }

        [HttpPut("completeUserTask/{userTaskOid}")]
        public void CompleteTask(Guid userTaskOid)
        {
            using (var cx = _dbFactory.CreateContext())
            {
                var userTask = cx.UserTasks.Where(t => t.UserTaskOid == userTaskOid).SingleOrDefault();
                if (userTask != null)
                {
                    userTask.CompleteDate = DateTime.UtcNow;
                    userTask.TaskStatusId = (int)TaskStatuses.Complete;
                    cx.SaveChanges();
                }
            }
        }

        [HttpPost("addUpdateUserTask")]
        public void AddUpdateUserTask(UserTaskToAdd userTaskDetails)
        {
            try
            {
                if (userTaskDetails.UserTaskOid == Guid.Empty)
                {
                    AddUserTask(userTaskDetails);
                }
                else
                {
                    UpdateUserTask(userTaskDetails);
                }
            }
            catch (Exception ex)
            {
                log.LogError(ex, "Could not login");
                var errorMsg = String.Format("Could not login, error: {0}", ex.Message);
                return BadRequest(errorMsg);
            }
        }

        [HttpDelete("userTask/{userTaskOid}")]
        public void DeleteUserTask(Guid userTaskOid)
        {
            using (var cx = _dbFactory.CreateContext())
            {
                var userTask = cx.UserTasks.Where(t => t.UserTaskOid == userTaskOid).SingleOrDefault();
                if (userTask != null)
                {
                    cx.UserTasks.Remove(userTask);
                    cx.SaveChanges();
                }
            }
        }

        private void UpdateUserTask(UserTaskToAdd userTaskDetails)
        {
            using (var cx = _dbFactory.CreateContext())
            {
                var userTask = cx.UserTasks.Where(t => t.UserTaskOid == userTaskDetails.UserTaskOid).SingleOrDefault();
                if (userTask != null)
                {
                    userTask.Name = userTaskDetails.Name;
                    userTask.OrderNumber = userTaskDetails.OrderNumber;
                    cx.SaveChanges();
                }
            }
        }

        private void AddUserTask(UserTaskToAdd userTaskDetails)
        {
            using (var cx = _dbFactory.CreateContext())
            {
                var userTask = new UserTask()
                {
                    UserTaskOid = Guid.NewGuid(),
                    UserOid = userTaskDetails.UserOid,
                    Name = userTaskDetails.Name,
                    OrderNumber = userTaskDetails.OrderNumber,
                    StartDate = DateTime.UtcNow,
                    TaskStatusId = (int)TaskStatuses.New,
                };
                cx.UserTasks.Add(userTask);
                cx.SaveChanges();
            }
        }
    }
}

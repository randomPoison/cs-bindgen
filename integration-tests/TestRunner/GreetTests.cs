using Xunit;

namespace TestRunner
{
    public class GreetTests
    {
        [Fact]
        public void GreetNumber()
        {
            string result = IntegrationTests.GreetANumber(7);
            Assert.Equal("Hello, #7!", result);
        }

        [Fact]
        public void GreetNumberRepeated()
        {
            for (int number = 0; number < 1000; number += 1)
            {
                string actual = IntegrationTests.GreetANumber(number);
                Assert.Equal($"Hello, #{number}!", actual);
            }
        }

        [Fact]
        public void ReturnNumber()
        {
            int result = IntegrationTests.ReturnANumber();
            Assert.Equal(7, result);
        }

        [Fact]
        public void StringArg()
        {
            string result = IntegrationTests.StringArg("Test");
            Assert.Equal("Hello, Test!", result);
        }

        [Fact]
        public void StringArgRepeated()
        {
            for (int number = 0; number < 1000; number += 1)
            {
                string result = IntegrationTests.StringArg("Test");
                Assert.Equal("Hello, Test!", result);
            }
        }

        [Fact]
        public void CreatePersonInfo()
        {
            using (PersonInfo info = new PersonInfo("David", 12))
            {
                Assert.Equal("David", info.Name());
                Assert.Equal(12, info.Age());
            }
        }

        [Fact]
        public void CreateManyPersonInfo()
        {
            for (int count = 0; count < 1000; count += 1)
            {
                PersonInfo info = new PersonInfo("Fred", 123);
                info.Dispose();
            }
        }

        [Fact]
        public void SetAge()
        {
            using (PersonInfo info = new PersonInfo("David", 12))
            {
                Assert.Equal(12, info.Age());
                info.SetAge(22);
                Assert.Equal(22, info.Age());
            }
        }

        [Fact]
        public void StaticFunction()
        {
            Assert.Equal(7, PersonInfo.StaticFunction());
        }

        [Fact]
        public void PersonAddress()
        {
            using (PersonInfo info = new PersonInfo("David", 12))
            {
                using (Address address = info.Address())
                {
                    Assert.Equal(123u, address.StreetNumber());
                    Assert.Equal("Cool Kids Lane", address.StreetName());
                }
            }
        }
    }
}
